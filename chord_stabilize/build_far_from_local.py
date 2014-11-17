"""
Mon Nov 17 10:24:04 IST 2014
by real.

A piece of code for constructing Far links from Local links.
Based on the Chord_stabilizing article.
"""
import random

NODE_NAMESPACE = 128
NODE_NAMESPACE_SIZE = 2**NODE_NAMESPACE


# A node in the DHT:
class Node():
    def __init__(self,node_name):
        # Keep node's name:
        self.node_name = node_name
        self.local_links = set()
        self.far_links = set()
        self.links = self.local_links.union(self.far_links)

# Simulation of a DHT:
class SimDHT():
    def __init__(self,n,k):
        # number of nodes:
        self.n = n
        # Ring connectivity constant:
        self.k = k
        # Empty map of nodes initially:
        self.nodes = {}

        # Create the list of random nodes:
        self.build_nodes()
        # Create initial k-ring links:
        self.k_ring()

    def rand_node_name(self):
        """
        Return a random name for a node.
        """
        return random.randrange(NODE_NAMESPACE_SIZE)

    def name_dist(self,x,y):
        """
        Find the name distance between two node names.
        d(x,y) is the distance from x to y going clockwise.
        """
        return (y - x) % NODE_NAMESPACE_SIZE 

    def build_nodes(self):
        """
        Generate self.n random nodes, and put them nicely in to self.nodes and
        self.nodes_sorted.
        """
        # Generate list of random nodes:
        nodes_lst = [self.rand_node_name() for i in range(self.n)]

        # Map node names to indices in sorted list:
        for i,node_name in enumerate(nodes_lst):
            self.nodes[node_name] = Node(node_name)

    def nceil(self,val):
        """
        Get the first node (clockwise) with a name bigger or equal to val.
        """
        return min(self.nodes,lambda z:self.name_dist(val,z))

    def nfloor(self,val):
        """
        Get the first node (Counter-Clockwise) with a name smaller or equal to
        val.
        """
        return min(self.nodes,lambda z:self.name_dist(z,val))


    def k_ring(self):
        """
        Link all the node as a k-linked ring.
        Every node will be connected to his close k neighbours (From both
        sides) on the ring.
        """
        def k_connect(ind,nodes_lst):
            """
            Connect a node in the list to all the k neighbouring nodes. If we
            get to the end of the list, we do a wraparound. (The list is
            cyclic).
            """
            nd = self.nodes[nodes_lst[ind]]

            # First direction:
            lnk_ind = ind
            for j in range(self.k):
                lnk_ind = (lnk_ind + 1) % len(nodes_lst)
                nd.local_links.add(nodes_lst[lnk_ind])

            # Second direction:
            lnk_ind = ind
            for j in range(self.k):
                lnk_ind = (lnk_ind - 1) % len(nodes_lst)
                nd.local_links.add(nodes_lst[lnk_ind])

            nd.links = set(nd.local_links)

        # Create a sorted list of nodes:
        nodes_lst = sorted(self.nodes.keys())

        # Iterate over all nodes in the list, and make sure each of them is
        # connected to the closest k neighbours from both sides:
        for ind,_ in enumerate(nodes_lst):
            k_connect(ind,nodes_lst)


    def stabilize(self,node_name):
        """
        Try improving immediate links by asking first level links.
        Done according to the Chord Stabilizing article.
        """
        # Get the node's class:
        nd = self.nodes[node_name]
        # Initialize the Known nodes set:
        known = set(nd.links)

        for ln in nd.links:
            known.update(self.nodes[ln].links)

        # Remove myself from the known set:
        known.discard(node_name)

        # Find the optimal local links:

        known_lst = list(known)
        nd.local_links = set()
        # Find "before nd" k best local links:
        known_lst.sort(key=lambda z:self.name_dist(z,node_name))
        nd.local_links.update(set(known_lst[:self.k]))
        # Find "after nd" k best local links:
        known_lst.sort(key=lambda z:self.name_dist(node_name,z))
        nd.local_links.update(set(known_lst[:self.k]))

        # Find optimal far links:
        nd.far_links = set()
        for j in range(NODE_NAMESPACE):
            # First direction:
            opt = min(known_lst,key=lambda z:self.name_dist(\
                    (node_name + (2**j)) % NODE_NAMESPACE_SIZE,z))
            nd.far_links.add(opt)

            # Second direction:
            opt = min(known_lst,key=lambda z:self.name_dist(\
                    z,(node_name - (2**j)) % NODE_NAMESPACE_SIZE))
            nd.far_links.add(opt)

        nd.links = nd.local_links.union(nd.far_links)

    def stabilize_iteration(self):
        """
        Invoke the Stabilize operation for all the nodes in the network.
        """
        for node_name in self.nodes:
            self.stabilize(node_name)

    def check_links_exact(self,node_name):
        """
        Check if all the links of node_name are exact. That means: They are the
        optimal possible links given all the nodes in the network.
        """
        nodes_lst = sorted(self.nodes)
        # Do not include the node itself:
        nodes_lst.remove(node_name)

        # Find the globally optimal local links for the node:
        optimal_local = set()
        # Find "before nd" k best local links:
        nodes_lst.sort(key=lambda z:self.name_dist(z,node_name))
        optimal_local.update(set(nodes_lst[:self.k]))
        # Find "after nd" k best local links:
        nodes_lst.sort(key=lambda z:self.name_dist(node_name,z))
        optimal_local.update(set(nodes_lst[:self.k]))


        # Find the globally optimal far links for the node:
        optimal_far = set()
        for j in range(NODE_NAMESPACE):
            # First direction:
            opt = min(nodes_lst,key=lambda z:self.name_dist(\
                    (node_name + (2**j)) % NODE_NAMESPACE_SIZE,z))
            optimal_far.add(opt)

            # Second direction:
            opt = min(nodes_lst,key=lambda z:self.name_dist(\
                    z,(node_name - (2**j)) % NODE_NAMESPACE_SIZE))
            optimal_far.add(opt)


        optimal_links = optimal_local.union(optimal_far)

        # Check if the optimal links found are exactly what's inside 
        if optimal_links == self.nodes[node_name].links:
            return True
        return False

    def check_done(self):
        """
        Check if the links are optimal for all the nodes in the DHT.
        """
        for node_name in self.nodes:
            if not self.check_links_exact(node_name):
                return False

        return True


    def count_iters(self):
        count_iters = 0
        is_done = False

        while not is_done:
            print(count_iters)
            count_iters += 1
            print("Stabilizing...")
            self.stabilize_iteration()
            print("Verifying...")
            is_done = self.check_done()

        return count_iters

def go():
    sd = SimDHT(n=2**9,k=3)
    num_iters = sd.count_iters()
    print("number of iterations: ",num_iters)


if __name__ == "__main__":
    go()
