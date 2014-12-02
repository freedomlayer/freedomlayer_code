# By real
# freedomlayer.org

# Some code two create two illustrations.
# One is of an erdos renyi random graph (Should be fast mixing), and the other
# one is of a 2 dimensional grid. It's an example for a graph that is not fast
# mixing.

import networkx as nx
import matplotlib.pyplot as plt

def draw_random_graph(i):
    """
    Draw a random graph with 2**i nodes,
    and p=i/(2**i)
    """
    g_random = nx.gnp_random_graph(2**i,i/(2**i))
    nx.draw_spring(g_random,node_size=20)
    plt.savefig("./random_graph.svg")
    plt.close()
    # plt.show()

def draw_grid_graph(m,n):
    g_grid = nx.grid_2d_graph(m,n)
    # nx.draw(g_grid,node_size=20)
    nx.draw_spectral(g_grid,node_size=20)
    plt.savefig("./grid.svg")
    plt.close()
    # plt.show()

def go():
    draw_random_graph(8)
    draw_grid_graph(20,20)


if __name__ == "__main__":
    go()
