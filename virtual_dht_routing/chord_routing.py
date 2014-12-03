"""
3.12.2014
by real

Testing the idea of routing in a mesh using a Virtual DHT. Inspired by
"Pushing Chord into the Underlay".
"""
import random
from collections import namedtuple

# Number of bits in ident number:
IDENT_BITS = 48

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


# A node:
class Node():
    def __init__(self,k,ident=None):
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
        # Initialize list of known nodes:
        # self.known = []

        self.neighbours = []
        self.best_succ = []
        self.best_pred = []

    def set_neighbours(self,knodes):
        """
        knodes to be the neighbours of this Node.
        """
        self.neighbours = []
        for kn in knodes:
            self.neighbours.append(\
                    kn._replace(path_len=1))

        # Update known nodes:
        self.add_known_nodes(0,self.neighbours)

    def add_known_nodes(self,source_path_len,knodes):
        """
        Add a set of known nodes to self.known .
        Take the change of path_len into acount.
        """
        # Update the path lengths:
        updated_knodes = [kn._replace(path_len=kn.path_len+source_path_len)\
                for kn in knodes]

        # Sort all notable known nodes lexicographically by virtual distance
        # from self.ident, and path length:
        pool = self.best_succ + self.best_pred + updated_knodes
        pool.sort(key=lambda kn:\
                (dist_ident(self.ident,kn.ident),kn.path_len))

        # We check if any change happened due to adding known nodes:
        changed = False

        # Set best successors and best predecessors found:
        changed = changed or (set(self.best_succ) != set(pool[:self.k]))
        self.best_succ = pool[:self.k]  # First self.k

        changed = changed or (set(self.best_pred) != set(pool[-self.k:]))
        self.best_pred = pool[-self.k:] # Last self.k

        return changed

    def get_known(self):
        """
        Return a list of all known nodes.
        Items in the list are unique.
        """
        pool = set()
        pool.update(self.neighbours)
        pool.update(self.best_succ)
        pool.update(self.best_pred)
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


# Simulation for a mesh network with Virtual DHT abilities:
class VirtualDHT():
    def __init__(self,n,k):
        self.k = k
        self.num_nodes = n

        # Generate nodes and neighbours links:
        self.gen_nodes()
        self.rand_neighbours()


    def gen_nodes(self):
        """
        Generate n nodes with random identity numbers.
        """
        self.nodes = []
        for i in range(self.num_nodes):
            self.nodes.append(Node(self.k))

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
        for nd in self.nodes:
            # Sample a set of indices (Which represent a set of nodes).
            # Those nodes will be nd's neighbours:
            nindices = random.sample(range(self.num_nodes),self.k)
            # Initialize a list of neighbours:
            nd.set_neighbours(map(self.make_knode,nindices))

    def iter_node(self,i):
        """
        Ask all known nodes for better known nodes.
        i is the index of the node in self.nodes.
        """
        changed = False
        nd = self.nodes[i]
        for kn in nd.get_known():
            kn_node = self.nodes[kn.lindex]
            res = nd.add_known_nodes(kn.path_len,kn_node.get_known())
            changed = changed or res
        return changed

    def iter_all(self):
        """
        Perform a full iteration, where all nodes ask other nodes for better
        nodes.
        """
        changed = False
        for i in range(self.num_nodes):
            res = self.iter_node(i)
            changed = changed or res

        return changed

    def converge(self,max_iters=0x10):
        """
        "converge" the DHT by iterating until nothing changes.
        """
        for i in range(max_iters):
            has_changed = self.iter_all()
            print(".")
            if not has_changed:
                break


    def sample_path_len(self,num_samp=0x100):
        """
        Find an approximated average to the path_len to successor and
        predecessor.
        """
        sum_succ_path = 0.0
        sum_pred_path = 0.0

        snodes = random.sample(self.nodes,num_samp)
        for sn in snodes:
            sum_succ_path += sn.get_best_succ().path_len
            sum_pred_path += sn.get_best_pred().path_len

        return sum_succ_path/num_samp,sum_pred_path/num_samp


def go():
    i = 8
    k = i
    n = 2**i
    vd = VirtualDHT(n,k)
    vd.converge()
    print(vd.sample_path_len())
    

if __name__ == "__main__":
    go()


