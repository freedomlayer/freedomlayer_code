extern crate rand;
extern crate petgraph;

use self::rand::{Rng};
use self::petgraph::graphmap;
use self::petgraph::algo::{kosaraju_scc, connected_components};

use network::{Network};
use std::hash::Hash;

use random_util::{choose_k_nums};
use std::collections::VecDeque;


/// Information of some node in the network about 
/// a local tower (Closest of a certain color).
#[derive(Clone)]
pub struct LocalTowerInfo {
    gateway: usize,
    distance: u64,
    tower_node: usize,
}

/// Choose nodes to be towers. We pick num_towers towers of every color. There are num_colors
/// different tower colors.
pub fn choose_towers<Node: Hash + Eq + Clone, R: Rng>(net: &Network<Node>, 
                  num_towers: usize, num_colors: usize, rng: &mut R) -> Vec<Vec<usize>> {

    let mut chosen_towers: Vec<Vec<usize>> = Vec::new();

    for _ in 0 .. num_colors {
        // Pick random towers for a certain color:
        let mut ctowers = choose_k_nums(num_towers, net.igraph.node_count(), rng)
            .into_iter()
            .collect::<Vec<usize>>();
        // Sort for determinism:
        ctowers.sort();
        chosen_towers.push(ctowers);
    }
    chosen_towers
}

/// Update operation: A given node is told about a path to a local tower.
struct UpdateOper {
    node: usize,
    tower_color: usize,
    tower_index: usize,
    local_tower_info: LocalTowerInfo,
}

fn init_towers_info(num_nodes: usize, num_colors: usize) ->
    Vec<Vec<Option<LocalTowerInfo>>> {
    let mut towers_info: Vec<Vec<Option<LocalTowerInfo>>> = Vec::new();
    for i in 0 .. num_nodes {
        towers_info.push(Vec::new());
        for _ in 0 .. num_colors {
            towers_info[i].push(None);
        }
    }
    towers_info
}

/// Converge information about local towers. 
/// Every node will learn about the closest local towers
/// of every color.
/// This function uses a lot of memory, and can not be run
/// for networks of size 2^16.
#[allow(dead_code)]
pub fn calc_towers_info_mem_heavy<Node: Hash + Eq + Clone>(net: &Network<Node>, 
    chosen_towers: &Vec<Vec<usize>>) -> Vec<Vec<Option<LocalTowerInfo>>> {

    let mut towers_info = init_towers_info(net.igraph.node_count(), 
                                           chosen_towers.len());

    let mut pending_opers: VecDeque<UpdateOper> = VecDeque::new();

    // Add initial update operations from all chosen towers.
    // Later the information about those towers will propagage all over the network.
    for tower_color in 0 .. chosen_towers.len() {
        for tower_index in 0 .. chosen_towers[tower_color].len() {
            let tower_node = chosen_towers[tower_color][tower_index];
            pending_opers.push_back(UpdateOper {
                node: tower_node,
                tower_color,
                tower_index,
                local_tower_info: LocalTowerInfo {
                    gateway: tower_node,
                    distance: 0,
                    tower_node,
                }
            });
        }
    }

    // Start handling pending operations:
    while let Some(oper) = pending_opers.pop_front() {
        let ltower_info_opt: &mut Option<LocalTowerInfo> = 
            &mut towers_info[oper.node][oper.tower_color];

        let should_update = match *ltower_info_opt {
            None => true, 
            Some(ref ltower_info) => {
                // Check if the new offered tower info (oper.local_tower_info) 
                // is better than the current one (ltower_info):
                if (ltower_info.distance, 
                    ltower_info.gateway, 
                    ltower_info.tower_node) >
                    (oper.local_tower_info.distance, 
                     oper.local_tower_info.gateway, 
                     oper.local_tower_info.tower_node) {
                    true
                } else {
                    false
                }
            }
        };

        if !should_update {
            continue
        }

        // Update local tower information:
        *ltower_info_opt = Some(oper.local_tower_info.clone());
        // Notify all neighbors about new information:
        for nei in net.igraph.neighbors(oper.node) {
            pending_opers.push_back(UpdateOper {
                node: nei,
                tower_color: oper.tower_color,
                tower_index: oper.tower_index,
                local_tower_info: LocalTowerInfo {
                    gateway: oper.node,
                    distance: oper.local_tower_info.distance + 1,
                    tower_node: oper.local_tower_info.tower_node,
                }
            });
        }
    }

    towers_info
}


/// Perform one iteration of calculating towers info.
/// Return whether any changed happen during this iteration.
fn iter_towers_info<Node: Hash + Eq + Clone>(net: &Network<Node>,
                 chosen_towers: &Vec<Vec<usize>>,
                 towers_info: &mut Vec<Vec<Option<LocalTowerInfo>>>) -> bool {

    let mut changed = false;

    for node in net.igraph.nodes() {
        for nei in net.igraph.neighbors(node) {
            for tower_color in 0 .. chosen_towers.len() {
                if towers_info[node][tower_color].is_none() {
                    continue
                }
                // This is the candidate LocalTowerInfo for nei:
                let mut candidate_info = towers_info[node][tower_color].clone().unwrap();
                candidate_info.distance += 1;
                candidate_info.gateway = node;
                // Current nei's LocalTowerInfo:

                if towers_info[nei][tower_color].is_none() {
                    changed = true;
                    towers_info[nei][tower_color] = Some(candidate_info);
                    continue
                }

                let nei_info = towers_info[nei][tower_color].clone().unwrap();

                if (candidate_info.distance,
                    candidate_info.gateway,
                    candidate_info.tower_node) <
                   (nei_info.distance,
                    nei_info.gateway,
                    nei_info.tower_node) {

                    changed = true;
                    towers_info[nei][tower_color] = Some(candidate_info);

                }
            }
        }
    }
    changed
}


/// Converge information about local towers. 
/// Every node will learn about the closest local towers
/// of every color.
pub fn calc_towers_info<Node: Hash + Eq + Clone>(net: &Network<Node>, 
    chosen_towers: &Vec<Vec<usize>>) -> Vec<Vec<Option<LocalTowerInfo>>> {

    let mut towers_info = init_towers_info(net.igraph.node_count(), 
                                           chosen_towers.len());

    // Add initial update operations from all chosen towers.
    // Later the information about those towers will propagage all over the network.
    for tower_color in 0 .. chosen_towers.len() {
        for tower_index in 0 .. chosen_towers[tower_color].len() {
            let tower_node = chosen_towers[tower_color][tower_index];
            let tower_info = &mut towers_info[tower_node][tower_color];
            *tower_info = Some(LocalTowerInfo {
                gateway: tower_node,
                distance: 0,
                tower_node,
            });
        }
    }

    while iter_towers_info(net, chosen_towers, &mut towers_info) {
    }

    towers_info
}

/// Make sure that all LocalTowerInfo fields are not None
pub fn is_towers_info_filled(towers_info: &Vec<Vec<Option<LocalTowerInfo>>>) -> bool {
    for node in 0 .. towers_info.len() {
        for tower_color in 0 .. towers_info[node].len() {
            if towers_info[node][tower_color].is_none() {
                return false;
            }
        }
    }
    return true;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct TowerGraphNode {
    tower_color: usize,
    tower_index: usize,
}

/// Check if overlay directed graph of towers is connected.
/// Returns (connected, strongly_connected)
pub fn is_connected(chosen_towers: &Vec<Vec<usize>>, 
        towers_info: &Vec<Vec<Option<LocalTowerInfo>>>) -> (bool, bool) {

    // An overlay directed graph of the towers in the network
    // and the connections between them.
    // A tower T is connected to a tower T' if T' is the closest tower to T of some color.
    let mut towers_graph: graphmap::DiGraphMap<usize,()> = 
        graphmap::DiGraphMap::new();

    // Add towers as nodes to the graph:
    for tower_color in 0 .. chosen_towers.len() {
        for tower_index in 0 .. chosen_towers[tower_color].len() {
            towers_graph.add_node(chosen_towers[tower_color][tower_index]);
        }
    }

    // For every tower, add all connections to closest local towers as nodes.
    let graph_nodes = towers_graph.nodes().collect::<Vec<usize>>();
    for tower_node in graph_nodes {
        for tower_color in 0 .. chosen_towers.len() {
            towers_graph.add_edge(tower_node, 
                  towers_info[tower_node][tower_color].clone().unwrap().tower_node,());
        }
    }

    let sconnected_comps = kosaraju_scc(&towers_graph);
    (connected_components(&towers_graph) == 1, sconnected_comps.len() == 1)
}



#[cfg(test)]
mod tests {
    extern crate rand;
    use super::*;
    use network_gen::gen_network;
    use self::rand::{StdRng};

    #[test]
    fn test_calc_towers_info() {
        // Generate a random network:
        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let net = gen_network(0, 7, 15, 1, 2, &mut rng);

        let chosen_towers = choose_towers(&net, 4, 16, &mut rng);
        let towers_info = calc_towers_info(&net, &chosen_towers);
        assert!(is_connected(&chosen_towers, &towers_info) == (true, true));
        assert!(is_towers_info_filled(&towers_info));

    }

}
