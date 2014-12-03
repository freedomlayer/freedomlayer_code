"""
3.12.2014
by real

Testing the idea of routing in a mesh using a Virtual DHT. Inspired by
"Pushing Chord into the Underlay".
"""
import random
import heapq
import bisect
from collections import namedtuple

# Number of bits in ident number:
IDENT_BITS = 20

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
    def __init__(self,k,fk,ident=None):
        """
        Initialize a node.
        """
        # If ident value is not specified, we randomize one:
        if ident is None:
            self.ident = rand_ident()
        else:
            self.ident = ident

        # Argument related to the maximum size of the num_known set:
        self.k = k

        # Argument related to amount of known best finger candidates.
        self.fk = fk

        # Initialize list of known nodes:
        # self.known = []

        self.neighbours = []
        self.best_succ = []
        self.best_pred = []
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


        # Update known nodes:
        self.add_known_nodes(0,self.neighbours)

    def add_known_best_succ(self,knodes):
        """
        Add a new list of knodes to self.best_succ
        The path_len is assumed to be correct.
        """
        # Sort all notable known nodes lexicographically by virtual distance
        # from self.ident, and path length:
        pool = remove_knodes_duplicates(self.best_succ + knodes)
        # Set best successors found:
        self.best_succ = heapq.nsmallest(self.k,pool,key=lambda kn:\
                (dist_ident(self.ident,kn.ident),kn.path_len))


    def add_known_best_pred(self,knodes):
        """
        Add a new list of knodes to self.best_pred
        The path_len is assumed to be correct.
        """
        pool = remove_knodes_duplicates(self.best_pred + knodes)
        # Set best predecessors found:
        self.best_pred = heapq.nsmallest(self.k,pool,key=lambda kn:\
                (dist_ident(kn.ident,self.ident),kn.path_len))

    def add_known_best_finger_succ(self,f,knodes):
        """
        If any of the nodes in knodes is a better candidate for the f's
        successor finger, we replace.
        """
        pool = remove_knodes_duplicates(self.best_finger_succ[f] + knodes)
        self.best_finger_succ[f] = heapq.nsmallest(self.fk,pool,key=lambda kn:\
                (dist_ident(self.get_finger_succ_loc(f),kn.ident),kn.path_len))

    def add_known_best_finger_pred(self,f,knodes):
        """
        If any of the nodes in knodes is a better candidate for the f's
        predecessor finger, we replace.
        """
        pool = remove_knodes_duplicates(self.best_finger_pred[f] + knodes)
        self.best_finger_pred[f] = heapq.nsmallest(self.fk,pool,key=lambda kn:\
                (dist_ident(kn.ident,self.get_finger_pred_loc(f)),kn.path_len))

    def add_known_nodes(self,source_path_len,knodes):
        """
        Add a set of known nodes to self.known .
        Take the change of path_len into acount.
        """
        # Update the path lengths:
        updated_knodes = [kn._replace(path_len=kn.path_len+source_path_len)\
                for kn in knodes]

        # Make sure the node self.ident is not inside:
        updated_knodes = list(filter(lambda kn:kn.ident != self.ident,\
                updated_knodes))

        self.add_known_best_succ(updated_knodes)
        self.add_known_best_pred(updated_knodes)

        for f in range(IDENT_BITS):
            self.add_known_best_finger_succ(f,updated_knodes)
            self.add_known_best_finger_pred(f,updated_knodes)


    def get_known(self):
        """
        Return a list of all known nodes.
        Items in the list are unique.
        """
        pool = set()
        pool.update(self.neighbours)
        pool.update(self.best_succ)
        pool.update(self.best_pred)

        for f in range(IDENT_BITS):
            pool.update(self.best_finger_succ[f])
            pool.update(self.best_finger_pred[f])
        return list(pool)

    def get_close(self):
        """
        Return a list of the closest known nodes.
        Close in the virtual sense, to self.ident,
        and to the possible fingers on the Chord DHT.
        """
        pool = set()

        pool.update(self.best_succ)
        pool.update(self.best_pred)

        for f in range(IDENT_BITS):
            pool.update(self.best_finger_succ[f])
            pool.update(self.best_finger_pred[f])

        return list(pool)

    def get_best_succ(self):
        """
        Get the best successor.
        """
        return min(self.best_succ,\
                key=lambda kn:dist_ident(self.ident,kn.ident))

    def get_best_pred(self):
        """
        Get the best predecessor.
        """
        return min(self.best_pred,\
                key=lambda kn:dist_ident(kn.ident,self.ident))

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
    def __init__(self,n,k,fk,nei):

        # Amount of nodes:
        self.num_nodes = n
        # Half amount of neighbours per node:
        self.nei = nei
        # Known nodes parameter:
        self.k = k
        # Known finger nodes parameter:
        self.fk = fk

        # Generate nodes and neighbours links:
        self.gen_nodes()
        self.rand_neighbours()


    def gen_nodes(self):
        """
        Generate n nodes with random identity numbers.
        """
        self.nodes = []
        for i in range(self.num_nodes):
            self.nodes.append(Node(self.k,self.fk))

    def make_knode(self,i,path_len=0):
        """
        Given an index i of a node in self.nodes,
        create a Knode tuple. Optionally set path_len.
        """
        return Knode(path_len=path_len,\
                ident=self.nodes[i].ident,\
                lindex=i)

    def rand_neighbours(self):
        """
        Randomize immediate neighbours links between the nodes.
        """
        # Initialize neighbours sets as empty sets:
        nodes_nei = [set() for _ in range(self.num_nodes)]

        for i,nd in enumerate(self.nodes):
            # Sample a set of indices (Which represent a set of nodes).
            # Those nodes will be nd's neighbours:
            nodes_nei[i].update(\
                    random.sample(range(self.num_nodes),self.nei))

            # Remove myself:
            nodes_nei[i].discard(i)

            # To make the graph undirected, we add i to be neighbour of all
            # i's neighbours:
            for j in nodes_nei[i]:
                nodes_nei[j].add(i)

        for i,nd in enumerate(self.nodes):
            # Initialize a list of neighbours:
            nd.set_neighbours(map(self.make_knode,list(nodes_nei[i])))

    def iter_node(self,i):
        """
        Ask all known nodes for better known nodes.
        i is the index of the node in self.nodes.
        """
        nd = self.nodes[i]
        for kn in nd.get_close():
        # for kn in nd.get_known():
        # for kn in nd.neighbours:
            kn_node = self.nodes[kn.lindex]
            nd.add_known_nodes(kn.path_len,kn_node.get_close())

    def iter_all(self):
        """
        Perform a full iteration, where all nodes ask other nodes for better
        nodes.
        """
        for i in range(self.num_nodes):
            self.iter_node(i)

    def converge(self,max_iters=0x10):
        """
        "converge" the DHT by iterating until nothing changes.
        """
        for i in range(max_iters):
            self.iter_all()
            print(".",end="",flush=True)
            if self.verify():
                print("\nReached correct succ and pred.")
                return

        print("\nmax_iters acheived.")

    def verify_succ_pred(self):
        """
        Verify the best successor and predecessor found for all nodes.
        """
        # Get all nodes (as Knodes), and sort them according to ident:
        lnodes = list(map(self.make_knode,range(self.num_nodes)))
        lnodes.sort(key=lambda ln:ln.ident)

        # Iterate all nodes, and make sure that the best succ and best pred
        # match the sorted array lnodes. Note that in lnodes, the best
        # successor of lnodes[i] is lnodes[i+1]. The best predecessor is
        # lnodes[i-1].
        for i,ln in enumerate(lnodes):
            nd = self.nodes[ln.lindex]
            succ_ident = lnodes[(i+1) % self.num_nodes].ident
            pred_ident = lnodes[(i-1) % self.num_nodes].ident

            if nd.get_best_succ().ident != succ_ident:
                return False

            if nd.get_best_pred().ident != pred_ident:
                return False

        return True

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
            
            for f in range(IDENT_BITS):
                ind = bisect.bisect_left(\
                        idents,nd.get_finger_succ_loc(f))
                f_succ = lnodes[(ind) % self.num_nodes]

                if nd.get_best_succ_finger(f).ident != f_succ.ident:
                    return False

                ind = bisect.bisect_right(\
                        idents,nd.get_finger_pred_loc(f))
                f_pred = lnodes[(ind-1) % self.num_nodes]

                if nd.get_best_pred_finger(f).ident != f_pred.ident:
                    return False


        return True

    def verify(self):
        """
        Verify all the found nodes.
        """
        if not self.verify_succ_pred():
            return False

        if not self.verify_succ_pred_fingers():
            return False

        return True

    def sample_path_len(self,num_samp=0x200):
        """
        Find an approximated average to the path_len to successor and
        predecessor.
        """
        sum_succ_path = 0.0
        sum_pred_path = 0.0

        # We don't want to sample more than the total amount of nodes:
        num_samp = min([num_samp,self.num_nodes])

        snodes = random.sample(self.nodes,num_samp)
        for sn in snodes:
            sum_succ_path += sn.get_best_succ().path_len
            sum_pred_path += sn.get_best_pred().path_len

        return sum_succ_path/num_samp,sum_pred_path/num_samp

def go():
    i = 7
    nei = i # amount of neighbours
    k = i   # Amount of known to keep.
    fk = 3
    n = 2**i
    print("i = ",i)
    vd = VirtualDHT(n,k=k,fk=fk,nei=nei)
    vd.converge(max_iters=8)
    print(vd.sample_path_len())
    print(vd.verify_succ_pred())
    

if __name__ == "__main__":
    go()


