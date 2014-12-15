"""
15.12.2014,
by real.

Checking some idea regarding relative addresses in a graph.
The address of every vertex in the graph will have some kind of a relative
address, which is dependend upon the names of the nodes around the specific
node.

"""

import random
import math
import hashlib
import struct


IDENT_SIZE = 40
MAX_IDENT = 2**IDENT_SIZE
K = 1

def gen_random_ident():
    """
    Generate random identity
    """
    return random.randrange(MAX_IDENT)

class Hashes(object):
    def __init__(self,k):
        self.k = k  # Number of different hashes.
        return

    def hashi(self,i,x):
        """
        Hash the value x using hash function number i.
        This is implemented in an ad-hoc way using the sha256 function.
        """
        tmp_str = str(i) + "AA" + str(x) + "BB" + str(i)
        tmp_bytes = tmp_str.encode('utf-8')
        sh = hashlib.sha256()
        sh.update(tmp_bytes)
        return struct.unpack("Q",sh.digest()[:8])[0]


    def hash_set(self,ss):
        """
        Turn a set into a list of hashes.
        """
        hashes = []
        for i in range(self.k):
            hashes.append(min(\
                    map(lambda x:self.hashi(i,x),ss)))
        return hashes

    def combine_hashes(self,hashes_list):
        """
        Combine a few set hashes into one hash.
        """
        t_hashes = zip(*hashes_list)
        hashes = []
        for hashes_i in t_hashes:
            hashes.append(min(hashes_i))

        return hashes

hashes = Hashes(K)


class Party(object):
    def __init__(self,max_layer):
        self.ident = gen_random_ident()

        self.max_layer = max_layer
        self.neighbours = set()
        self.layers = [None for i in range(max_layer)]
        self.is_advers = False

        return

class ComGraph(object):
    def __init__(self,num_parties,num_advers,max_layer,degree):
        self.num_parties = num_parties
        self.num_advers = num_advers
        assert self.num_advers == 0 # For this version.
        assert self.num_parties > self.num_advers
        self.max_layer = max_layer
        self.degree = degree
        self.parties = {}

        self.gen_parties()
        self.gen_connections()
        self.init_wave()
        self.wave_iterations()

    def gen_parties(self):
        for i in range(self.num_parties):
            p = Party(self.max_layer)
            if i < self.num_advers:
                p.is_advers = True
            self.parties[p.ident] = p

        self.parties_nums = list(self.parties.keys())
        return

    def gen_connections(self):
        # We divide by two because there are incoming and outgoing connections:
        num_connect = self.degree // 2 
        for p_num in self.parties:
            rand_parties = set(random.sample(self.parties_nums,num_connect))
            rand_parties.discard(p_num)
            for p_num_to in rand_parties:
                # Add links in both directions:
                self.parties[p_num].neighbours.add(p_num_to)
                self.parties[p_num_to].neighbours.add(p_num)

        return

    def init_wave(self):
        for p_num in self.parties:
            p = self.parties[p_num]
            # Add myself as the first hash in layer 0:
            p.layers[0] = hashes.hash_set([p.ident])
            # Add my neighbours as the next hash in layer 1:
            # p.layers[1] = \
            #     hashes.hash_set(set(p.neighbours).union(set([p.ident])))
        return


    def combine_generic(self,hash_lists):
        """
        Combine all hashes list.
        If any list is None, do not consider it.
        If all the lists are None, then return None.
        """
        new_hl_list = []
        for hl in hash_lists:
            if hl is None:
                continue
            new_hl_list.append(hl)

        if len(new_hl_list) > 0:
            return hashes.combine_hashes(new_hl_list)
        else:
            return None


    def wave_iterate(self):
        for p_num in self.parties:
            p = self.parties[p_num]

            # Update p's layers:
            for lay in range(0,self.max_layer-1):
                layers_list = []

                for ne_num in p.neighbours:
                    ne = self.parties[ne_num]
                    layers_list.append(ne.layers[lay])

                # now, Add my own previous layer: (To avoid periodicity
                # problems)
                layers_list.append(p.layers[lay])

                # Finally, combine everything:
                p.layers[lay+1] = self.combine_generic(layers_list)

        return

    def wave_iterations(self):
        for i in range(self.max_layer+1):
            self.wave_iterate()
        return

    def get_landmarks(self,layers):
        # We mark distances to any landmark we have seen on our way.
        landmarks = dict()
        for lay in range(self.max_layer):
            if layers[lay] is None:
                break

            for i in range(hashes.k):
                cur_land = (i,layers[lay][i])
                if cur_land not in landmarks:
                    landmarks[cur_land] = lay
        return landmarks


    def random_pair(self):
        """
        Generate a random pair of parties (a,b), that are not equal.
        """
        return random.sample(self.parties_nums,2)

    def get_mediator(self,a,b):
        pa = self.parties[a]
        pb = self.parties[b]

        la = self.get_landmarks(pa.layers)
        lb = self.get_landmarks(pb.layers)

        # Get the intersection of known mediators:
        inter = set(la.keys()).intersection(set(lb.keys()))

        assert len(inter) > 0

        def path_dist(med):
            """
            Length of path obtained by mediator.
            """
            return la[med] + lb[med]

        return min(inter,key=path_dist)

    def measure_highest_load(self,tries):
        med_found = {}
        for i in range(tries):
            a,b = self.random_pair()
            med = self.get_mediator(a,b)
            try:
                med_found[med] += 1
            except KeyError:
                med_found[med] = 1

        max_elem = max(med_found.items(),key=lambda x:x[1])
        return max_elem[1]


def test_load():
    for j in range(7,17):
        num_parties = 2**j
        num_advers = 0
        max_layer = j
        degree = j

        tries = 10000

        cg = ComGraph(num_parties,num_advers,max_layer,degree)
        load = cg.measure_highest_load(tries)
        print("2^" + str(j),"|",load,"/",tries)

    return

if  __name__ == "__main__":
    test_load()

