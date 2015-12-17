
# 15.12.2015
# Implementation of network coordinates, based on Bourgain's theorem for Metric
# embedding.


import math
import random
import networkx as nx

import hashlib



def is_in_set(x,i,j,seed=0):
    """
    Check if node x is inside the set A_ij
    Generally a node x has probability 2^{-j} of being inside A_ij
    """
    m = hashlib.md5()
    src_data = '||{}||{}||{}||{}||'.format(x,i,j,seed).encode('ASCII')
    m.update(src_data)
    res = m.digest()

    num_bytes = math.ceil(j / 8)
    # Extract num_bytes bytes from res, which are a bit more than j bits.
    val = int.from_bytes(res[:num_bytes],'little')
    # Remove a few bits, so that we get exactly j bits:
    val >>= (8 - (j % 8))
    if val == 0:
        # This will happen with probability 2^{-j}
        return True


    return False



# Bourgain's coordinates calculation class

class BCoords:
    def __init__(self,g,seed=0):
        
        # Make sure that the graph is big enough:
        if len(g) <= 1:
            raise RuntimeError('Given graph contains just 1 or 0 nodes!')

        # Make sure that the given graph is connected:
        if not nx.is_connected(g):
            raise RuntimeError('The given graph is not connected!')

        self._seed = seed       # Sets choice random seed
        self._g = g             # Keep graph.
        self._n = len(self._g)  # Number of graph nodes.

        # Calculate the required amount of coordinates, according to Bourgain's
        # theorem:
        self._t = math.ceil(math.log2(self._n))
        self._m = 18*math.ceil(math.log2(self._n))
        # self._m = 144*math.ceil(math.log2(self._n))

        # Initialize coordinates for all nodes:
        self._calc_coords()

    def _init_coords(self):
        """
        Initialize coordinates for all nodes.
        Coordinates that are not known are marked with None.
        """
        self._coords = [None for i in range(len(self._g))]
        for node in self._g:
            self._coords[node] = \
                    [[None for j in range(self._t)] for i in range(self._m)]

            # Find out which sets contain node, and mark proximity 0 from those
            # sets:
            for i in range(self._m):
                for j in range(self._t):
                    if is_in_set(node,i,j,self._seed):
                        self._coords[node][i][j] = 0


    def _iter_coords_calc(self):
        """
        Perform one iteration of updating all coordinates for nodes.
        Every nodes asks his neighbors for their coordinates, and updates his
        own coordinates accordingly.
        """
        has_changed = False # Has anything changed after this iteration?

        for node in self._g:
            for i in range(self._m):
                for j in range(self._t):
                    for nei in self._g.neighbors(node):
                        if self._coords[nei][i][j] is None:
                            continue

                        if self._coords[node][i][j] is None:
                            self._coords[node][i][j] = \
                                    self._coords[nei][i][j] + 1
                            has_changed = True
                            continue

                        if self._coords[node][i][j] > \
                                self._coords[nei][i][j] + 1:
                            self._coords[node][i][j] = \
                                    self._coords[nei][i][j] + 1
                            has_changed = True

        return has_changed


    def _calc_coords(self):
        """
        Calculate Bourgain coordinates for all nodes
        """
        print('Initializing coordinates...')
        self._init_coords()
        print('Iterating coordinates calculation')
        has_changed = True
        while has_changed:
            has_changed = self._iter_coords_calc()
            print('Iter...')


    def get_bdistance(self,x,y):
        """
        Get distance according to Bourgain's coordinates between two nodes on
        the graph: x and y.
        """

        res = 0

        for i in range(self._m):
            for j in range(self._t):
                assert (self._coords[x][i][j] is None) == \
                    (self._coords[y][i][j] is None)

                if (self._coords[x][i][j] == None) and \
                        (self._coords[y][i][j] == None):
                            continue

                res += abs(self._coords[x][i][j] - self._coords[y][i][j])

        return res/(self._m * self._t)

    def get_mdistance(self,x,y):
        """
        Get maximum style distance.
        Rely on the fact that |d(x,S) - d(y,S)| <= d(x,y)
        so calculate Max |d(x,S) - d(y,S)|
        """
        max_res = 0
        for i in range(self._m):
            for j in range(self._t):
                if (self._coords[x][i][j] == None) and \
                        (self._coords[y][i][j] == None):
                            diff = 0
                else:
                    diff = abs(self._coords[x][i][j] - self._coords[y][i][j])

                if max_res < diff:
                    max_res = diff

        return max_res




def test_is_in_set():
    """
    Test if is_in_set works properly (If produces correct probabilities).
    """
    trials = 10000

    count = 0
    j = 5
    for i in range(trials):
        if is_in_set(1,i,j,seed=0):
            count += 1

    assert abs((count/trials) - 2**(-j)) < (1/math.sqrt(trials))


    count = 0
    j = 9
    for i in range(trials):
        if is_in_set(1,i,j,seed=1):
            count += 1

    assert abs((count/trials) - 2**(-j)) < (1/math.sqrt(trials))



def go():
    l = 12
    n = 2**l
    p = 3*math.log(n)/n
    g = nx.fast_gnp_random_graph(n,p)

    bc = BCoords(g,seed=5)

    for i in range(64):
        a,b = random.sample(g.nodes(),2)
        # Get usual graph distance:
        g_dist = len(nx.shortest_path(g,source=a,target=b)) - 1
        # Get Bourgain's distance:
        b_dist = bc.get_bdistance(a,b)
        # Get maximum style distance:
        m_dist = bc.get_mdistance(a,b)

        print('{} {} : g_dist: {}, b_dist: {}, m_dist: {}'.\
                format(a,b,g_dist,b_dist,m_dist))
    


if __name__ == '__main__':
    go()

