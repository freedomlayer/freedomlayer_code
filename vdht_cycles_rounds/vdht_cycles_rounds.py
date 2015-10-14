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

        # Keep the graph:
        self._g = g

        # Calculate the highest possible l such that 2^{l-1} < len(g):
        self._l = int(math.log2(len(g) - 1)) + 1

        # Build fingers structure:
        self._fingers = {node:dict() for node in self._g}
        for node in self._g:
            # Adding 0 finger:
            self._fingers[node][0] = Finger()

            # Adding negative fingers:
            for j in range(self._l):
                f = -(2**j)
                self._fingers[node][f] = Finger()

            # Adding positive fingers:
            for j in range(self._l):
                f = 2**j
                self._fingers[node][f] = Finger()
                
        res = self.alg_iter()
        if not res:
            raise AlgStateError('First iteration did not make a change '
                    'to the fingers. Aborting.')

    def alg_iter(self):
        """
        Perform one iteration of the algorithm.
        Return True if something has changed (Since the last state). Returns
        False otherwise (We have reached stationary state).
        """
        # Has anything changed during this iteration?
        changed = False

        for node in self._g:
            # Get all known nodes to node:
            known = self._get_known(node)

            for f in self._fingers[node]:
                # Absolute location of finger:
                loc = (node + f) % len(self._g)

                # Update left side:
                bl = self._best_l(loc,known)
                if not self._fingers[node][f].l == bl:
                    self._fingers[node][f].l = bl
                    changed = True

                # Update right side:
                br = self._best_r(loc,known)
                if not self._fingers[node][f].r == br:
                    self._fingers[node][f].r = br
                    changed = True

        return changed

    def _get_known(self,node):
        """
        Get all known nodes to a node using:
        - His initial neighbors on the graph.
        - The nodes he maintains on his fingers.
        - All the fingers and initial neighbors of his finger nodes.
        """

        known = set()
        # Add graph neighbors:
        known.update(self._g.neighbors(node))

        # Add fingers:
        for f,fin in self._fingers[node]:
            if fin.l is not None:
                known.add(fin.l)
            if fin.r is not None:
                known.add(fin.r)

        # For each finger node, add his fingers and graph neighbors:
        for fnode in self._fingers[node]:
            # Add the neighbors of fnode:
            known.update(self._g.neighbors(fnode))

            for f,fin in self._fingers[fnode]:
                if fin.l is not None:
                    known.add(fin.l)
                if fin.r is not None:
                    known.add(fin.r)

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

