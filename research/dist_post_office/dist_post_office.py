"""
16.12.2014
by real

Checking load in the Distributed Post Office.
"""

import math
import random
import heapq
import bisect
from collections import namedtuple

import networkx as nx

# A named tuple for Known node:
# path_len is the path length source node,
# ident is the identity value of the Known node.
# lindex is the list index of the Known node.
Knode = namedtuple('Knode', ['path_len', 'ident','lindex'])

def hashi(i,x):
    """
    Hash the value x using hash function number i.
    This is implemented in an ad-hoc way using the sha256 function.
    """
    tmp_str = str(i) + "AA" + str(x) + "BB" + str(i)
    tmp_bytes = tmp_str.encode('utf-8')
    sh = hashlib.sha256()
    sh.update(tmp_bytes)

    # Take the first 8 bytes of the result:
    return struct.unpack("Q",sh.digest()[:8])[0]


# A node:
class Node():
    def __init__(self,dpo,ind,ident=None):
        """
        Initialize a node.
        """
        # Pointer to distributed post office
        self.dpo = dpo

        # If ident value is not specified, we randomize one:
        if ident is None:
            self.ident = self.dpo.rand_ident()
        else:
            self.ident = ident

        # Index inside the list of nodes:
        self.ind = ind

        # Initialize list of known nodes:
        self.neighbours = []

        # Initialize specials:
        # ( self.specials[hash_func_index][distance] )
        for hi in range(self.dpo.num_hashes):
            self.specials[hi] = [None for j in range(self.dpo.max_distance)]



    def set_neighbours(self,knodes):
        """
        
        """
        assert False



# Simulation for the Distributed Post office
class DPostOffice():
    def __init__(self,graph,num_hashes,ident_bits):


        # The network graph we are going to use:
        self.graph = graph

        # Assert that the graph is connected:
        assert nx.is_connected(self.graph)

        # Max layer will be the diameter of the graph:
        self.max_distance = nx.diameter(self.graph)

        # Amount of nodes:
        self.num_nodes = self.graph.number_of_nodes()

        # Amount of bits in identity:
        self.ident_bits = ident_bits

        # Maximum size of identity:
        self.max_ident = 2**self.ident_bits

        # Evade the birthday paradox:
        assert (self.num_nodes ** 2.5) <= self.max_ident

        # Amount of cryptographic hash functions to be used:
        self.num_hashes = num_hashes

        # Generate nodes and neighbours links:
        self.gen_nodes()
        self.install_neighbours()

    def rand_ident(self):
        """
        Generate random identity in the range [0,self.max_ident)
        """
        return random.randrange(self.max_ident)

    def dist_ident(self,x,y):
        """
        Distance between two nodes (According to ident):
        """
        return (y - x) % self.max_ident


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
    i = 11      # Parameter for graph generation.
    ident_bits = math.ceil(i*2.6)
    fk = i
    print("||| i =",i)
    print("||| ident_bits =",ident_bits)

    print("Generating graph...")
    # g = gen_grid_graph(i)
    g = gen_gnp_graph(i)
    print("Generating Network...")
    vd = DPostOffice(graph=g, ident_bits=ident_bits)

    print("Initiating convergence...\n")
    vd.converge(max_iters=0x80)

if __name__ == "__main__":
    go()


