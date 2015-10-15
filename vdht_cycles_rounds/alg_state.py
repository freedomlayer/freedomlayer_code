# 14.10.2015 
# by real

# A program that tests the following theorem: Given a stationary state of the
# IterF algorithm, every cycle for nodes is of exactly one round.

import math
import networkx as nx


class AlgStateError(Exception): pass

# A structure to maintain a finger:
class Finger:
    def __init__(self):
        # Left:
        self.l = None
        # Right
        self.r = None


# A structure for one node:
class Node:
    def __init__(self):
        self.fingers = None
        # Temporary recipients for the next update message to send.
        self.temp_recipients = set()


# A state in the algorithm:
class AlgState:
    def __init__(self,g):
        """
        g -- A connected graph
        """
        # Make sure that the graph is big enough:
        if len(g) <= 1:
            raise AlgStateError('Given graph contains just 1 or 0 nodes!')

        # Make sure that the given graph is connected:
        if not nx.is_connected(g):
            raise AlgStateError('The given graph is not connected!')

        # Get the maximum value of a node in the graph:
        max_node_val = max([node for node in g])

        # Make sure that the node values are numeric, and that no node value is
        # larger than the amount of nodes in the graph.
        if max_node_val >= len(g):
            raise AlgStateError('Invalid node numbers.')

        # Keep the graph:
        self._g = g

        # Calculate the highest possible l such that 2^{l-1} < len(g):
        self._l = int(math.log2(len(g) - 1)) + 1


        # Build a dictionary of nodes:
        self._nodes = {node:Node() for node in self._g}

        # Build fingers structure:
        self._fingers = {node:dict() for node in self._g}
        for node in self._g:
            self._nodes[node].fingers = dict()

            # Adding 0 finger:
            # self._nodes[node].fingers[0] = Finger()

            # Adding negative fingers:
            for j in range(self._l):
                f = -(2**j)
                self._nodes[node].fingers[f] = Finger()

            # Adding positive fingers:
            for j in range(self._l):
                f = 2**j
                self._nodes[node].fingers[f] = Finger()
                
        res = self.alg_iter()
        if not res:
            raise AlgStateError('First iteration did not make a change '
                    'to the fingers. Aborting.')

    def next_node(self,node):
        """
        Return the "next node": The right node of finger 0.
        """
        return self._nodes[node].fingers[1].r

    def _process_update_msg(self,rnode,snode,uset):
        """
        Node rnode processes and update message with set uset, sent from snode
        Returns True is anything has changed in rnode's fingers. Returns False
        otherwise.
        """

        # Add snode to rnode's temp_recipients, if rnode doesn't know snode.
        # This way rnode will send to snode a set of his known nodes in the
        # next algorithm iteration.
        if snode not in self._get_known(rnode):
            self._nodes[rnode].temp_recipients.add(snode)


        # Add snode to the set of nodes, to build an extended update set:
        euset = set(uset)
        euset.add(snode)

        # Has anything changed in rnode's fingers?
        changed = False

        # Update left side:
        for f in self._nodes[rnode].fingers:
            loc = (rnode + f) % len(self._g)

            bl = self._best_l(loc,euset)

            cur_l = self._nodes[rnode].fingers[f].l
            if cur_l is None:
                self._nodes[rnode].fingers[f].l = bl
            else:
                if self._d(bl,rnode) < self._d(cur_l,rnode):
                    assert cur_l != bl
                    self._nodes[rnode].fingers[f].l = bl
                    changed = True


        # Update right side:
        for f in self._nodes[rnode].fingers:
            loc = (rnode + f) % len(self._g)

            br = self._best_r(loc,euset)

            cur_r = self._nodes[rnode].fingers[f].r
            if cur_r is None:
                self._nodes[rnode].fingers[f].r = br
            else:
                if self._d(rnode,br) < self._d(rnode,cur_r):
                    assert cur_r != br
                    self._nodes[rnode].fingers[f].r = br
                    changed = True

        return changed


    def _time_tick(self,node):
        """
        A time tick has occured for node node.
        node will send update messages to all his known nodes, and set the
        temp_recipients set to be empty.

        Returns if any node has changed his fingers state. True for change,
        False for no change.
        """
        # Has any node changed his fingers state?
        changed = False

        known = self._get_known(node)
        for rnode in known:
            res = self._process_update_msg(rnode,node,known)
            changed = changed or res

        # Clear node's temp_recipients:
        self._nodes[node].temp_recipients.clear()

        return changed


    def alg_iter(self):
        """
        Perform one iteration of the algorithm.
        Return True if something has changed (Since the last state). Returns
        False otherwise (We have reached stationary state).
        """
        # Has anything changed during this iteration?
        changed = False

        for node in self._g:
            res = self._time_tick(node)
            changed = changed or res

        return changed


    def run_until_stat(self):
        """
        Iterate until a stationary state is acheived.
        """
        changed = True
        while True:
            prev_changed = changed
            changed = self.alg_iter()
            # We only break if we have two consecutive iterations where a
            # change didn't happen.
            if (not changed) and (not prev_changed):
                break


    def _get_known(self,node):
        """
        Get all known nodes to a node using:
        - His initial neighbors on the graph.
        - The nodes he maintains on his fingers.
        - Temporary recipients
        """

        known = set()
        # Add graph neighbors:
        known.update(self._g.neighbors(node))

        # Add fingers:
        for fin in self._nodes[node].fingers.values():
            if fin.l is not None:
                known.add(fin.l)
            if fin.r is not None:
                known.add(fin.r)

        # Add temporary recipients:
        known.update(self._nodes[node].temp_recipients)

        # Make sure that node is not included in the known set:
        known.discard(node)

        return known

    def _d(self,x,y):
        """
        Calculate cyclic distance between x and y (Modulo mod)
        """
        return (y - x) % len(self._g)

    def _best_l(self,loc,nset):
        """
        Closest node from the left.
        Get the node from nset that minimizes d(z,loc)
        """
        return min(nset,key=lambda z:self._d(z,loc))

    def _best_r(self,loc,nset):
        """
        Closest node from the right.
        Get the node from nset that minimizes d(loc,z)
        """
        return min(nset,key=lambda z:self._d(loc,z))

    ##########[Some Checks]#################################################

    def is_next_node_injection(self):
        """
        Make sure that the next_node function is injective. (This means:
        next_node(x) = next_node(y) ==> x = y)
        """
        # A set of nodes range:
        nrange = set()
        for node in self._g:
            nnode = self.next_node(node)
            # If the value is already in the range, next_node is not injective.
            if nnode in nrange:
                return False
            # Add the value to the range:
            nrange.add(nnode)

        return True

    def _iter_cycle_reps(self):
        """
        Iterate through the cycles of AlgState. Every time yield a node that
        represents a cycle.
        """
        nodes = set([node for node in self._g])
        
        # Continue as long as there are remaining nodes:
        while len(nodes) > 0:
            # Get any node x from the remaining nodes set:
            x = min(nodes)

            # Iterate through the cycle for node x. Remove all the nodes from the
            # cycle:
            z = self.next_node(x)
            nodes.remove(z)
            while z != x:
                z = self.next_node(z)
                nodes.remove(z)

            yield x


    def _is_one_round_cycle(self,node):
        """
        Check if a node is inside a one round cycle.
        """
        # The total distance sum of the cycle:
        dist_sum = 0

        # Walk through the cycle that begins with x, until we meet x again.
        # During the walk, we sum the distance.
        x = node
        z = x
        z_next = self.next_node(x)
        dist_sum += self._d(z,z_next)
        
        # Continue until we reach x again:
        while z_next != x:
            z = z_next
            z_next = self.next_node(z)
            dist_sum += self._d(z,z_next)

        # We expect at least one round:
        assert dist_sum >= len(self._g)
        
        if dist_sum != len(self._g):
            return False

        return True


    def is_only_one_cycle(self):
        """
        Check if there is just one cycle.
        """
        # Count the amount of cycle representatives. If it is 1, return True.
        # Otherwise return False
        return sum(1 for _ in self._iter_cycle_reps()) == 1


    def is_all_cycles_one_round(self):
        """
        Check if all the cycles are of one round.
        """
        for node in self._iter_cycle_reps():
            if not self._is_one_round_cycle(node):
                return False

        return True


###########################################################################
###########################################################################


def go():
    """
    Run the IterF algorithm on some random graphs.
    """
    l = 7
    n = 2**l
    p = 3*math.log(n)/n

    for i in range(100):
        # Generate a random graph:
        g = nx.fast_gnp_random_graph(n,p)
        # Build AlgState from the graph:
        algs = AlgState(g)
        # Run the IterF algorithm until we reach a stationary state:
        algs.run_until_stat()

        # Print the properties of the resulting state:
        print('next_node Injective: {} | '
              'One round cycles: {} | '
              'Is only one cycle: {}'.\
                      format(\
                algs.is_next_node_injection(),\
                algs.is_all_cycles_one_round(),\
                algs.is_only_one_cycle())\
                )

if __name__ == '__main__':
    go()
