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

# Fingers we are interested in:
SUCC_FINGERS = [0]
PRED_FINGERS = [0]

# SUCC_FINGNERS = list(range(IDENT_BITS))
# PRED_FINGERS = list(range(IDENT_BITS))

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
    def __init__(self,nodes,ind,fk,ident=None):
        """
        Initialize a node.
        """
        # If ident value is not specified, we randomize one:
        if ident is None:
            self.ident = rand_ident()
        else:
            self.ident = ident

        # Argument related to amount of known best finger candidates.
        self.fk = fk

        # The list of all nodes:
        self.nodes = nodes

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


        # Update known nodes:
        # self.add_known_nodes(0,self.neighbours)

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

        for f in SUCC_FINGERS:
            self.add_known_best_finger_succ(f,updated_knodes)
        for f in PRED_FINGERS:
            self.add_known_best_finger_pred(f,updated_knodes)

        new_close = set(self.get_close())
        new_known = set(self.get_known())

        # Knode tuple of myself:
        myself = Knode(path_len=0,ident=self.ident,lindex=self.ind)

        for kn in set(updated_knodes).union(old_known):
            assert kn.ident != myself.ident

            best_succ_to_kn = min(new_known,key=lambda nkn:
                   (dist_ident(kn.ident,nkn.ident),nkn.path_len))
            best_pred_to_kn = min(new_known,key=lambda nkn:
                   (dist_ident(nkn.ident,kn.ident),nkn.path_len))

            def d(x,y):
                return min([dist_ident(x,y),dist_ident(y,x)])

            closest_to_kn = min(new_known,key=lambda nkn:
                    (d(nkn.ident,kn.ident),nkn.path_len))

            # kn was just added: (It was not there before):
            if kn in new_close.difference(old_close):

                # (destination_index,path_len,list_of_nodes)
                # print("=============================")
                # print("improve ",self.ind)
                # print(kn.ident)

                # queue.append((kn.lindex,kn.path_len,[myself] + \
                #     self.get_known()))

                # queue.append((kn.lindex,kn.path_len,[myself]))

                # for nkn in new_known.difference(set([kn])):
                #     queue.append((nkn.lindex,nkn.path_len,[kn]))

                # queue.append((kn.lindex,kn.path_len,[myself] + \
                #     self.get_known()))

                # queue.append((kn.lindex,kn.path_len,[closest_to_kn]))

                continue

            # kn was not included (We got better nodes), or removed:
            if ((kn in updated_knodes) and (kn not in new_close)) or\
                    (kn in old_close.difference(new_close)):

                # queue.append((kn.lindex,kn.path_len,[best_succ_to_kn,best_pred_to_kn]))
                queue.append((kn.lindex,kn.path_len,[closest_to_kn]))
                # print("other ",self.ident,"-->",kn.ident,":",best_succ_to_kn.ident,best_pred_to_kn.ident)

                # print("other ",self.ident,"-->",kn.ident,":",closest_to_kn.ident,\
                #              "(",d(self.ident,kn.ident))

                #   print("------")
                #   print("other ",self.ind)
                #   print(kn.lindex)
                #   print(best_succ_to_kn.lindex,best_pred_to_kn.lindex)
                continue

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
        for f in SUCC_FINGERS:
            pool.update(self.best_finger_succ[f])
        for f in PRED_FINGERS:
            pool.update(self.best_finger_pred[f])
        return list(pool)

    def get_close(self):
        """
        Return a list of the closest known nodes.
        Close in the virtual sense, to self.ident,
        and to the possible fingers on the Chord DHT.
        """
        pool = set()

        for f in SUCC_FINGERS:
            pool.update(self.best_finger_succ[f])
        for f in PRED_FINGERS:
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
    def __init__(self,n,fk,nei):

        # Amount of nodes:
        self.num_nodes = n
        # Half amount of neighbours per node:
        self.nei = nei
        # Known finger nodes parameter:
        self.fk = fk

        # Generate nodes and neighbours links:
        self.gen_nodes()
        self.rand_neighbours()
        # Assert that the graph is connected:
        assert nx.is_connected(self.graph)


    def gen_nodes(self):
        """
        Generate n nodes with random identity numbers.
        """
        self.nodes = []
        for i in range(self.num_nodes):
            self.nodes.append(Node(self.nodes,i,self.fk))

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

        p = self.nei / self.num_nodes
        self.graph = nx.fast_gnp_random_graph(self.num_nodes,p)

        for i,nd in enumerate(self.nodes):
            # Sample a set of indices (Which represent a set of nodes).
            # Those nodes will be nd's neighbours:
            nodes_nei[i].update(nx.neighbors(self.graph,i))

            # Make sure that i is not his own neighbour:
            assert i not in nodes_nei[i]

        for i,nd in enumerate(self.nodes):
            # Initialize a list of neighbours:
            nd.set_neighbours(map(self.make_knode,list(nodes_nei[i])))


    def iter_node(self,i):
        """
        push a notification for all the known nodes about our set of known
        nodes. i is the index of the node in the self.nodes list.
        """
        assert False
        nd = self.nodes[i]
        nd.add_known_nodes(0,nd.get_known_nodes())
        # nd.add_known_nodes(0,nd.neighbours)

    def iter_all(self):
        """
        Perform a full iteration, where all nodes ask other nodes for better
        nodes.
        """
        # (destination_index,path_len,list_of_nodes)
        count = 0
        queue = []
        # Initialize queue:
        for i in range(self.num_nodes):
            nd = self.nodes[i]
            # queue.append((i,0,nd.neighbours))
            nd.notify_all(queue)

        while len(queue) > 0:
            # print(len(queue))
            destination_index,path_len,list_of_nodes = queue.pop(0) # dfs
            count += 1
            # queue.pop(0) for bfs
            nd = self.nodes[destination_index]
            nd.add_known_nodes(path_len,list_of_nodes,queue)

        print("Total queue count: ",count)

    def converge(self,max_iters=0x10):
        """
        "converge" the DHT by iterating until nothing changes,
        or max_iters is acheived.
        """
        for i in range(max_iters):
            self.iter_all()
            print(".",end="",flush=True)
            print("Verified: ",self.verify())
            print("avg path len:",self.sample_path_len())
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
            
            for f in SUCC_FINGERS:
                ind = bisect.bisect_left(\
                        idents,nd.get_finger_succ_loc(f))
                f_succ = lnodes[(ind) % self.num_nodes]

                if nd.get_best_succ_finger(f).ident != f_succ.ident:
                    return False

            for f in PRED_FINGERS:
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
            for f in SUCC_FINGERS:
                sum_finger_path += sn.get_best_succ_finger(f).path_len
            for f in PRED_FINGERS:
                sum_finger_path += sn.get_best_pred_finger(f).path_len

        num_fingers = len(SUCC_FINGERS) + len(PRED_FINGERS)
        return sum_finger_path/(num_samp * num_fingers)

def go():
    print("SUCC_FINGERS: ",SUCC_FINGERS)
    print("PRED_FINGERS: ",PRED_FINGERS)
    for i in range(6,9):
        print("i =",i)
        # nei = i # amount of neighbours
        nei = i
        fk = 1
        n = 2**i
        vd = VirtualDHT(n,fk=fk,nei=nei)
        vd.converge(max_iters=0x5)
        print("Verify result:",vd.verify())
        print(vd.sample_path_len())
    

if __name__ == "__main__":
    go()


