"""
7.12.2014
by real

Testing the idea of routing in a mesh using a Virtual DHT. Inspired by
"Pushing Chord into the Underlay".
"""
import random
import heapq
import bisect
from collections import namedtuple

import networkx as nx

# Number of bits in ident number:
IDENT_BITS = 40

# Maximum possible identity value.
# Note that this value isn't really the maximum. It is maximum + 1.
MAX_IDENT = 2**IDENT_BITS

# A named tuple for Known node:
# path_len is the path length source node,
# ident is the identity value of the Known node.
# lindex is the list index of the Known node.
Knode = namedtuple('Knode', ['path_len', 'ident','lindex'])


def rand_ident():
    """
    Generate random identity in the range [0,MAX_IDENT)
    """
    return random.randrange(MAX_IDENT)

def dist_ident(x,y):
    """
    Distance between two nodes (According to ident):
    """
    return (y - x) % MAX_IDENT

def remove_knodes_duplicates(knodes):
    """
    Go over a list of knodes, and remove knodes that show up more than once.
    In case of node ident showing more than once, we pick the shorter path.
    """
    if len(knodes) == 0:
        return knodes

    knodes.sort(key=lambda kn:(kn.ident,kn.path_len))

    # Resulting array
    cur_ident = knodes[0].ident
    res = [knodes[0]]
    for kn in knodes[1:]:
        if kn.ident != cur_ident:
            cur_ident = kn.ident
            res.append(kn)

    return res


# A node:
class Node():
    def __init__(self,vdht,ind,ident=None):
        """
        Initialize a node.
        """
        # If ident value is not specified, we randomize one:
        if ident is None:
            self.ident = rand_ident()
        else:
            self.ident = ident

        # Pointer to virtual DHT:
        self.vdht = vdht

        # Argument related to amount of known best finger candidates.
        self.fk = self.vdht.fk

        # The list of all nodes:
        self.nodes = self.vdht.nodes

        # Index inside the list of nodes:
        self.ind = ind

        # Initialize list of known nodes:
        self.neighbours = []
        self.best_finger_succ = [list() for f in range(IDENT_BITS)]
        self.best_finger_pred = [list() for f in range(IDENT_BITS)]

    def get_finger_succ_loc(self,f):
        """
        Get the exact location of successor finger f.
        """
        return (self.ident + 2**f) % MAX_IDENT

    def get_finger_pred_loc(self,f):
        """
        Get the exact location of predecessor finger f.
        """
        return (self.ident - 2**f) % MAX_IDENT

    def set_neighbours(self,knodes):
        """
        set knodes to be the neighbours of this Node.
        """
        self.neighbours = []
        for kn in knodes:
            # Make sure we don't have ourselves as a neighbour:
            if kn.ident == self.ident:
                continue
            # A neighbour has a path length 1:
            self.neighbours.append(\
                    kn._replace(path_len=1))


    def add_known_best_finger_succ(self,f,knodes):
        """
        If any of the nodes in knodes is a better candidate for the f's
        successor finger, we replace.
        """
        pool = remove_knodes_duplicates(self.neighbours + self.best_finger_succ[f] + knodes)
        self.best_finger_succ[f] = heapq.nsmallest(self.fk,pool,key=lambda kn:\
                (dist_ident(self.get_finger_succ_loc(f),kn.ident),kn.path_len))

    def add_known_best_finger_pred(self,f,knodes):
        """
        If any of the nodes in knodes is a better candidate for the f's
        predecessor finger, we replace.
        """
        pool = remove_knodes_duplicates(self.neighbours + self.best_finger_pred[f] + knodes)
        self.best_finger_pred[f] = heapq.nsmallest(self.fk,pool,key=lambda kn:\
                (dist_ident(kn.ident,self.get_finger_pred_loc(f)),kn.path_len))


    def add_known_nodes(self,source_path_len,knodes,queue):
        """
        Add a set of known nodes to self.known .
        Take the change of path_len into account.
        """

        if self.vdht.cmsg:
            # Keep old set of close nodes:
            old_close = set(self.get_close())
            old_known = set(self.get_known())

        # Update the path lengths:
        updated_knodes = [kn._replace(path_len=kn.path_len+source_path_len)\
                for kn in knodes]

        # Make sure the node self.ident is not inside:
        updated_knodes = list(filter(lambda kn:kn.ident != self.ident,\
                updated_knodes))

        # If there are no new real nodes, there is nothing to do here:
        if len(updated_knodes) == 0:
            return

        for f in self.vdht.dht_succ_fingers:
            self.add_known_best_finger_succ(f,updated_knodes)
        for f in self.vdht.dht_pred_fingers:
            self.add_known_best_finger_pred(f,updated_knodes)

        if self.vdht.cmsg:
            new_close = set(self.get_close())
            new_known = set(self.get_known())

            # Knode tuple of myself:
            myself = Knode(path_len=0,ident=self.ident,lindex=self.ind)

            for kn in set(updated_knodes).union(old_known):
                assert kn.ident != myself.ident

                def d(x,y):
                    return min([dist_ident(x,y),dist_ident(y,x)])

                closest_to_kn = min(new_known,key=lambda nkn:
                        (d(nkn.ident,kn.ident),nkn.path_len))

                # kn was not included (We got better nodes), or removed:
                if ((kn in updated_knodes) and (kn not in new_close)) or\
                        (kn in old_close.difference(new_close)):

                    queue.append((kn.lindex,kn.path_len,[closest_to_kn]))


    def notify_all(self,queue):
        """
        Notify all known nodes about my known nodes:
        """

        my_known = self.get_known()
        # Knode tuple of myself:
        myself = Knode(path_len=0,ident=self.ident,lindex=self.ind)

        for kn in my_known:
            queue.append((kn.lindex,kn.path_len,my_known + [myself]))


    def get_known(self):
        """
        Return a list of all known nodes.
        Items in the list are unique.
        """
        pool = set()

        # Add neighbours:
        pool.update(self.neighbours)

        # Add fingers:
        for f in self.vdht.dht_succ_fingers:
            pool.update(self.best_finger_succ[f])
        for f in self.vdht.dht_pred_fingers:
            pool.update(self.best_finger_pred[f])
        return list(pool)

    def get_close(self):
        """
        Return a list of the closest known nodes.
        Close in the virtual sense, to self.ident,
        and to the possible fingers on the Chord DHT.
        """
        pool = set()

        for f in self.vdht.dht_succ_fingers:
            pool.update(self.best_finger_succ[f])
        for f in self.vdht.dht_pred_fingers:
            pool.update(self.best_finger_pred[f])

        return list(pool)

    def get_best_succ_finger(self,f):
        """
        Get the best successor for finger f.
        """
        return min(self.best_finger_succ[f],\
                key=lambda kn:dist_ident(self.get_finger_succ_loc(f),kn.ident))


    def get_best_pred_finger(self,f):
        """
        Get the best predecessor for finger f.
        """
        return min(self.best_finger_pred[f],\
                key=lambda kn:dist_ident(kn.ident,self.get_finger_pred_loc(f)))


# Simulation for a mesh network with Virtual DHT abilities:
class VirtualDHT():
    def __init__(self,graph,fk,cmsg,dht_fingers):

        # Known finger nodes parameter:
        self.fk = fk

        # The network graph we are going to use:
        self.graph = graph

        # Assert that the graph is connected:
        assert nx.is_connected(self.graph)

        # Amount of nodes:
        self.num_nodes = self.graph.number_of_nodes()

        # Send connectivity messages?
        self.cmsg = cmsg

        # Load fingers to be used in the Chord DHT:
        self.dht_succ_fingers,self.dht_pred_fingers = dht_fingers

        # Generate nodes and neighbours links:
        self.gen_nodes()
        self.install_neighbours()



    def gen_nodes(self):
        """
        Generate n nodes with random identity numbers.
        """
        self.nodes = []
        for i in range(self.num_nodes):
            self.nodes.append(Node(self,i))

    def make_knode(self,i,path_len=0):
        """
        Given an index i of a node in self.nodes,
        create a Knode tuple. Optionally set path_len.
        """
        return Knode(path_len=path_len,\
                ident=self.nodes[i].ident,\
                lindex=i)

    def install_neighbours(self):
        """
        Install the neighbours information inside the Nodes classes.
        """
        # Initialize neighbours sets as empty sets:
        nodes_nei = [set() for _ in range(self.num_nodes)]

        # Build translation tables between graph nodes
        # and node numbers 1 .. self.num_nodes
        self.graph_to_vec = {}
        self.vec_to_graph = []
        
        for i,gnd in enumerate(self.graph.nodes_iter()):
            self.vec_to_graph.append(gnd)
            self.graph_to_vec[gnd] = i

        for i,nd in enumerate(self.nodes):
            # Sample a set of indices (Which represent a set of nodes).
            # Those nodes will be nd's neighbours:
            gneighbours = nx.neighbors(self.graph,self.vec_to_graph[i])
            nodes_nei[i].update(map(lambda gnei:\
                    self.graph_to_vec[gnei],gneighbours))

            # Make sure that i is not his own neighbour:
            assert i not in nodes_nei[i]

        for i,nd in enumerate(self.nodes):
            # Initialize a list of neighbours:
            nd.set_neighbours(map(self.make_knode,list(nodes_nei[i])))


    def iter_all(self):
        """
        Perform a full iteration, where all nodes ask other nodes for better
        nodes.
        """
        # (destination_index,path_len,list_of_nodes)
        queue = []
        # Initialize queue:
        for i in range(self.num_nodes):
            nd = self.nodes[i]
            nd.notify_all(queue)

        while len(queue) > 0:
            destination_index,path_len,list_of_nodes = queue.pop() # bfs
            # queue.pop(0) for bfs
            nd = self.nodes[destination_index]
            nd.add_known_nodes(path_len,list_of_nodes,queue)

    def converge(self,max_iters=0x10):
        """
        "converge" the DHT by iterating until nothing changes,
        or max_iters is acheived.
        """

        # Print a table for real time results:
        header_f = "{0:12s} | {1:16s} | {2:18s} | {3:16s}"
        entry_f = "{0:12d} | {1:16s} | {2:18s} | {3:16f}"
        header = header_f.format("iter number","Fingers verified",\
                "path_len verified","avg path_len")
        print(header)
        print(len(header) * "-")

        for i in range(max_iters):
            self.iter_all()

            fingers_verify = str(self.verify())
            path_len_verify = str(self.verify_path_len())
            avg_path_len = self.sample_path_len()

            print(entry_f.format(i,fingers_verify,path_len_verify,avg_path_len))

            #   print("@@ Iter number ",i)
            #   print("Fingers verified: ",self.verify())
            #   print("path_len verified:",self.verify_path_len())
            #   print("Average path_len:",self.sample_path_len())
            #   print()

            # if self.verify():
            #     print("\nReached correct succ and pred + fingers.")
            #     return

        print("\nmax_iters acheived.")


    def verify_succ_pred_fingers(self):
        """
        Verify the succ and pred fingers found for all nodes.
        """
        # Get all nodes (as Knodes), and sort them according to ident:
        lnodes = list(map(self.make_knode,range(self.num_nodes)))
        lnodes.sort(key=lambda ln:ln.ident)
        idents = [ln.ident for ln in lnodes]

        for i,ln in enumerate(lnodes):
            nd = self.nodes[ln.lindex]
            
            for f in self.dht_succ_fingers:
                ind = bisect.bisect_left(\
                        idents,nd.get_finger_succ_loc(f))
                f_succ = lnodes[(ind) % self.num_nodes]

                if nd.get_best_succ_finger(f).ident != f_succ.ident:
                    return False

            for f in self.dht_pred_fingers:
                ind = bisect.bisect_right(\
                        idents,nd.get_finger_pred_loc(f))
                f_pred = lnodes[(ind-1) % self.num_nodes]

                if nd.get_best_pred_finger(f).ident != f_pred.ident:
                    return False

        return True

    def verify_path_len(self):
        """
        Make sure that the finger links are optimal in length.
        (That they are the shortest paths possible)
        """

        # Iterate over all nodes:
        for nd in self.nodes:
            for f in self.dht_succ_fingers:
                best_succ_f = nd.get_best_succ_finger(f)
                # Calculate shortest path on graph:
                spath_len = nx.shortest_path_length(self.graph,\
                        self.vec_to_graph[nd.ind],\
                        self.vec_to_graph[best_succ_f.lindex])

                # Check if the path we have to best_succ_f equals exactly
                # spath_len:
                if best_succ_f.path_len != spath_len:
                    return False

            for f in self.dht_pred_fingers:
                best_pred_f = nd.get_best_pred_finger(f)
                # Calculate shortest path on graph:
                spath_len = nx.shortest_path_length(self.graph,\
                        self.vec_to_graph[nd.ind],\
                        self.vec_to_graph[best_pred_f.lindex])

                # Check if the path we have to best_pred_f equals exactly
                # spath_len:
                if best_pred_f.path_len != spath_len:
                    return False

        return True


    def verify(self):
        """
        Verify all the found nodes.
        """
        if not self.verify_succ_pred_fingers():
            return False

        return True

    def sample_path_len(self,num_samp=0x200):
        """
        Find an approximated average to the path_len to successor and
        predecessor.
        """
        sum_finger_path = 0.0

        # We don't want to sample more than the total amount of nodes:
        num_samp = min([num_samp,self.num_nodes])

        snodes = random.sample(self.nodes,num_samp)
        for sn in snodes:
            for f in self.dht_succ_fingers:
                sum_finger_path += sn.get_best_succ_finger(f).path_len
            for f in self.dht_pred_fingers:
                sum_finger_path += sn.get_best_pred_finger(f).path_len

        num_fingers = len(self.dht_succ_fingers) + len(self.dht_pred_fingers)
        return sum_finger_path/(num_samp * num_fingers)

def gen_grid_graph(i):
    """
    Generate a grid graph with about 2**i nodes.
    """
    n = 2**i
    sn = int(n**(1/2))
    # Redefine amount of nodes:
    return nx.grid_2d_graph(sn,sn)

def gen_gnp_graph(i):
    """
    Generate a gnp random graph with 2**i nodes.
    """
    n = 2**i
    p = 2*i / (2**i)
    return nx.fast_gnp_random_graph(n,p)

def go():
    print("----------------------------")

    # Fingers we are interested in:
    # succ_fingers = [0]
    # pred_fingers = [0]
    succ_fingers = list(range(IDENT_BITS))
    pred_fingers = list(range(IDENT_BITS))
    cmsg = False # Connectivity messages.
    i = 7
    fk = i//2
    print("||| i =",i)
    print("||| fk =",fk)
    print("||| succ_fingers = ",succ_fingers)
    print("||| pred_fingers = ",pred_fingers)

    # g = gen_grid_graph(i)
    print("Generating graph...")
    g = gen_grid_graph(i)
    # g = gen_gnp_graph(i)
    print("Generating Network...")
    vd = VirtualDHT(graph=g,fk=fk,cmsg=cmsg,\
            dht_fingers=(succ_fingers,pred_fingers))

    print("Initiating convergence...\n")
    vd.converge(max_iters=0x80)
    

if __name__ == "__main__":
    go()


