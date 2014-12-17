"""
17.12.2014
by real

Checking out Landmarks navigation with random walk.
"""

import networkx as nx
import math
import random

import numpy as np

#############[Array operations]######################

def avg(lst):
    """
    Calculate the average of an array of numbers.
    """
    assert len(lst) > 0,"List must contain something"
    return sum(lst)/len(lst)

def max_var(lst):
    """
    Calculate variance of an array of numbers
    """
    lst_avg = avg(lst)

    vals = []
    for x in lst:
         vals.append(abs(x - lst_avg))

    return max(vals)

def median(lst):
    """
    Calculate the median of an array of numbers.
    """
    assert len(lst) > 0,"List must contain something"
    return sorted(lst)[len(lst)//2]

def normalize_prob(lst):
    """
    Normalize a list of numbers so that the sum will be 1.
    """
    total_sum = sum(lst)
    assert total_sum > 0,"sum should be positive"
    return [x/total_sum for x in lst]

############################################################

def geo_graph(m,n,d):
    """
    Generate a grid graph of size mXn.
    Then connect every node to d random nodes in the graph.
    """
    # Begin with a grid:
    res_graph = nx.grid_2d_graph(m,n,True)

    # Next connect each node to random nodes:
    for x in res_graph.nodes_iter():
        neis = random.sample(res_graph.nodes(),d)
        # We dont want to connect to ourselves:
        try:
            neis.remove(x)
        except ValueError:
            pass

        # Add the additional edges:
        for nei in neis:
            if not res_graph.has_edge(x,nei):
                res_graph.add_edge(x,nei)

    return res_graph



class GraphCoords(object):
    def __init__(self,num_nodes,k):
        # k is the number of landmarks (Special nodes).
        self.k = k
        self.num_nodes = num_nodes
        assert k < self.num_nodes
        self.graph = None
        self.landmarks = set()
        self.dists = {}

        self.gen_graph()
        self.calc_dists()
        return

    def gen_graph(self):
        ## Probability to have an edge:
        p = 2*math.log(self.num_nodes) / self.num_nodes
        ## Create a G(n,p) random graph:
        self.graph = nx.fast_gnp_random_graph(self.num_nodes,p)

        # sn = int(math.sqrt(self.num_nodes))
        # ln = 2*int(math.log(self.num_nodes))
        # self.graph = nx.grid_2d_graph(sn,sn,periodic=True)
        # self.graph = nx.connected_watts_strogatz_graph(self.num_nodes,2,0.5)

        # self.graph = geo_graph(sn,sn,3)
        
        self.landmarks = random.sample(self.graph.nodes(),self.k)

        return

    def calc_dists(self):
        """
        Calculate all distances from every landmark to all the parties in the
        graph.
        """
        for ld in self.landmarks:
            lengths = nx.single_source_dijkstra_path_length(self.graph,ld)
            for nd in self.graph.nodes():
                self.dists[(ld,nd)]=lengths[nd]
        return

    def get_coord(self,nd):
        """
        Get "coordinate" for a node nd.
        """
        coord = []
        for ld in self.landmarks:
            coord.append(self.dists[(ld,nd)])
        return tuple(coord)

    def random_pair(self):
        """
        Return a random pair of nodes in the graph.
        """
        return self.random_k(2)

    def random_k(self,k):
        """
        Return a random pair of nodes in the graph.
        """
        return random.sample(self.graph.nodes(),k)

    def calc_dist(self,n1,n2):
        """
        Calculate real distance between two nodes in the graph.
        Probably done using Dijkstra
        """
        return nx.shortest_path_length(self.graph,n1,n2)

    def all_diffs(self,x,y):
        """
        Calculate all diffs of the form |d(a,y) - d(a,x)| where a is a
        landmark.
        """
        def d(ld,x):
            """
            Distance between a landmark and a vertex.
            """
            return self.dists[(ld,x)]

        return [abs(d(ld,x) - d(ld,y)) \
                for ld in self.landmarks]

    def all_sums(self,x,y):
        """
        Calculate all the sums of the form d(a,y) + d(a,x) where a is a
        landmark.
        """
        def d(ld,x):
            """
            Distance between a landmark and a vertex.
            """
            return self.dists[(ld,x)]
        
        return [d(ld,x) + d(ld,y) \
                for ld in self.landmarks]

    def obs_dist(self,x,y):
        """
        Observed distance is a new metric that represents a different kind of
        distance between vertices.
        It is:
        Avg{ld \in landmarks}{|d(ld,x) - d(ld,y)|}
        """
        return avg(self.all_diffs(x,y))

    def obs_max_dist(self,x,y):
        """
        Calculates min{ld \in landmarks}{|d(ld,x) - d(ld,y)|}
        """
        return max(self.all_diffs(x,y))

    def obs_min_sum(self,x,y):
        """
        Calculates max({ld \in landmarks}{d(ld,x) + d(ld,y)}
        """
        return min(self.all_sums(x,y))

    def obs_sum(self,x,y):
        """
        Calculates avg({ld \in landmarks}{d(ld,x) + d(ld,y)}
        """
        return avg(self.all_sums(x,y))

    def random_walk(self,src,dst,base=150):
        """
        Try to travel from src to dst
        The "closest" neighbor to dst has the highest probability.
        """
        def e(x,y):
            """
            Distance metric
            """
            return (self.obs_max_dist(x,y),)
            #   return (self.obs_max_dist(x,y),\
            #           self.obs_min_sum(x,y),\
            #           self.obs_dist(x,y))
            # return self.obs_max_dist(x,y)

        def num_prob(cur_dist,new_dist):
            """
            Generate a weight for moving from cur_dist to new_dist
            """
            return base**(cur_dist[0] - new_dist[0])


        num_hops = 0
        x = src
        cur_dist = e(x,dst)

        while x != dst:
        # while cur_dist > (EPSILON,EPSILON):
            # print(x,dst,cur_dist) # debug
            neighbours = []
            for nei in self.graph.neighbors_iter(x):
                # Append neighbour and obs distance:
                neighbours.append((nei,e(nei,dst)))

            # Get probabilities according to inverse of obs_distance.
            # We use EPSILON as a defense to divison by zero.
            nums = [num_prob(cur_dist,dist) for (nei,dist) in neighbours]
            probs = normalize_prob(nums)
            # probs = normalize_prob([(1/(x + EPSILON)) for x in dists])
            # Choose the next neighbour. Neighbours which are closer to dst
            # Get a better probability to be chosen.
            index = np.random.choice(len(neighbours),p=probs)
            x,dist = neighbours[index]
            cur_dist = dist

            num_hops += 1

        return True,num_hops


def test_random_walk():
    for i in range(11,16):
        num_nodes = 2**i
        k = int(num_nodes**(1/2))
        # k = 2*i
        print("---------")
        print("i = ",i)
        gc = GraphCoords(num_nodes,k)
        print("Graph was generated.")

        hops_list = []
        for i in range(60):
            x,y = gc.random_pair()
            # print("Trying to reach...")
            res,num_hops = gc.random_walk(x,y)
            hops_list.append(num_hops)

        print("Average amount of hops: ",avg(hops_list))

if __name__ == "__main__":
    test_random_walk()
