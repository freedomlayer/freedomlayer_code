"""
16.12.2014
by real

Checking load in the Distributed Post Office.
"""

import math
import struct
import random
import collections

import hashlib

import networkx as nx

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

        # Initialize list of neighbours:
        self.neighbours = None

        # Initialize specials dictionary:
        self.specials_dc = None

        # Initialize specials:
        # ( self.specials[hash_func_index][distance] )
        self.specials = []
        for hi in range(self.dpo.num_hashes):
            self.specials.append([None for j in range(self.dpo.max_distance+1)])

        # Set my own index as the best at all hash functions for distance 0,
        # As I am the only node of distance 0 from myself.
        for hi in range(self.dpo.num_hashes):
            self.specials[hi][0] = self.ind


    def add_node(self,ind,distance,hash_i):
        """
        Check if the node with index "ind" and distance "distance" is better
        than the current node at self.specials[hash_i][distance].
        """

        # If there was not yet a candidate for distance "distance" and hash
        # index hash_i, we choose this node.
        if self.specials[hash_i][distance] is None:
            self.specials[hash_i][distance] = ind
            return

        # If there is something in self.specials[hash_i][distance], we choose
        # the "higher" out of the two:
        cur_ind = self.specials[hash_i][distance]

        self.specials[hash_i][distance] = max([cur_ind,ind],key=lambda j:\
                hashi(hash_i,self.dpo.nodes[j]))


    def set_neighbours(self,nei_nodes):
        """
        Initialize set of neighbours for this node.
        """
        # Set the neighbours list to be nei_nodes:
        self.neighbours = list(nei_nodes)


    def install_specials_dc(self):
        """
        Create a dictionary that maps knowns specials (By list index) to their
        distance. Save it as self.specials_dc.
        """

        # We only install this once:
        assert self.specials_dc is None

        # Initialize specials dictionary:
        self.specials_dc = {}

        for hi in range(self.dpo.num_hashes):
            for dist in range(self.dpo.max_distance+1):
                ind = self.specials[hi][dist]
                # All the specials should be filled by nodes from the network
                assert ind is not None
                self.specials_dc[ind] = dist


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
            nd.set_neighbours(nodes_nei[i])


    def calc_specials(self):
        """
        All nodes iterate together to calculate the special nodes for all the
        hash functions and all distances until self.max_distance.
        """

        assert self.max_distance >= 1

        # Iterate over all distances, starting from 0:
        for dist in range(self.max_distance):
            # Iterate over all nodes:
            for nd in self.nodes:
                # Iterate over all hash functions:
                for hi in range(self.num_hashes):
                    # Iterate over all nd's neighbours:
                    for nei_ind in nd.neighbours:
                        # Update nd's special at a specific distance and hash
                        # function according to information from it's
                        # neighbour nei_ind:
                        nei = self.nodes[nei_ind]
                        nd.add_node(nei.specials[hi][dist],dist+1,hi)

                    # Get special from nd's previous distance:
                    # We do this because the highest node of distance 5 might
                    # be also the highest node of distance 6, for example.
                    nd.add_node(nd.specials[hi][dist],dist+1,hi)


        # Install the specials_dc dictionary. This structure will be useful for
        # finding mediators later.
        for nd in self.nodes:
            nd.install_specials_dc()

    def get_best_mediator(self,a_ind,b_ind):
        """
        Find a best mediator node for the nodes a an b.
        A best mediator for a and b is a node that is both inside a.specials_dc
        and b.specials_dc, and it has the lowest sum of distances from a and b.
        """

        a = self.nodes[a_ind]
        b = self.nodes[b_ind]

        sa = a.specials_dc
        sb = b.specials_dc

        # intersection of known specials of a and b.
        # This gives us all the possible mediators between a and b:
        mediators = list(set(sa.keys()).intersection(set(sb.keys())))
        
        # The set of mediators is expected to be nonempty, as the "highest" of
        # every hash function should be both inside sa and sb:
        assert len(mediators) > 0

        # Create a random permutation of the same size of mediators.
        # This is done to randomize the choice of mediator, in case there are a
        # few mediators of the same quality.
        rperm = list(range(len(mediators)))
        random.shuffle(rperm)

        def path_dist(i):
            """
            Length of path obtained by mediator.
            """
            return (sa[mediators[i]] + sb[mediators[i]],\
                    rperm[i])

        # Return some mediator that gives shortest path:
        med_ind = min(range(len(mediators)),key=path_dist)
        return mediators[med_ind]
        # return min(mediators,key=path_dist)

    def measure_load(self,tries):
        """
        Try to pass a message through many random pairs of nodes, and count the
        mediators used to pass those messages. returns cnt_med: A counter of
        the mediators used.
        """
        cnt_med = collections.Counter()

        for i in range(tries):
            # Get a random pair of nodes:
            a_ind,b_ind = random.sample(range(self.num_nodes),2)
            med = self.get_best_mediator(a_ind,b_ind)
            cnt_med[med] += 1

        return cnt_med


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
    ########[Parameters]########################
    i = 12      # Parameter for graph generation.
    ident_bits = math.ceil(i*2.6)
    num_hashes = 1

    # Number of most common mediators to show:
    num_most_common = 0x10
    # Number of messages to simulate delivery:
    num_mess = 0x10000
    ############################################

    print("||| i =",i)
    print("||| num_hashes =",num_hashes)
    print("||| ident_bits =",ident_bits)

    print("Generating graph...")
    g = gen_grid_graph(i)
    # g = gen_gnp_graph(i)

    print("Generating Network...")
    dp = DPostOffice(graph=g,\
            num_hashes=num_hashes,\
            ident_bits=ident_bits)

    print("Calculating specials...")
    # Calculate specials for every node in the network:
    dp.calc_specials()

    print("Simulating" ,num_mess, "messages delivery...")
    cnt_med = dp.measure_load(num_mess)

    print("\nmost common mediators:\n")
    # Print header:
    header_ln = " {med:15s} | {ratio:10s} | {times:15s} ".format(\
            med="mediator index",ratio="ratio",\
            times="messages routed")
    print(header_ln)
    print("-" * len(header_ln))

    # Print table lines:
    for med,times in cnt_med.most_common(num_most_common):
        ln = " {med:15d} | {ratio:10f} | {times:15d} ".format(\
                med=med,ratio=times/num_mess,times=times)
        print(ln)



if __name__ == "__main__":
    go()


