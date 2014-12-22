"""
22.12.2014
by real
http://www.freedomlayer.org

We route messages in the network by random walking from source to destination.
We use the information from the network coordinates to make the random walk a
bit smarter.

This code measures the network load that occures as a result of the random
walks. For each node x, we count how many times a message was routed through x.
Finally we print the nodes with the highest amount of messages routed through
them. (Note that a message might visit one specific node more than once.)

If a node is a landmark, it is printed with asterisks around its number.
"""

import graph_coords

def measure_load():
    # Amount of messages to simulate:
    num_messages = 0x1000
    # Amount of most common visited nodes to show:
    num_most_common = 0x20
    # Parameter related to the size of the graph:
    i = 14
    # Function used to generate the graph:
    gen_graph_func = graph_coords.gen_gnp_graph
    # Base used to choosing weights for neighbours in random walk:
    base=0x16

    print("||| graph generation func =",gen_graph_func.__name__)
    print("||| i =",i)
    print("||| num_messages =",num_messages)
    print()

    k = i**2
    # Generate graph:
    print("Generating graph...")
    g = gen_graph_func(i)
    # Generate coordinates:
    print("Generating coordinates...")
    gc = graph_coords.GraphCoords(graph=g,k=k)
    # Simulate Delivery of num_messages messages:
    print("Simulating messages delivery...")
    avg_num_hops,cnt_visited_nodes =\
            gc.get_avg_num_hops(num_messages,base=base)

    # Extract the most visited node with respect to random walks:
    max_node,max_mess = cnt_visited_nodes.most_common(1)[0]

    print("\nmost commonly visited nodes:\n")
    # Print header:
    header_ln = " {node:15s} | {times:15s} ".format(\
            node="node", times="Times visited")
    print(header_ln)
    print('-' * len(header_ln))

    def node_str(node):
        """
        generate a string to represent a node.
        If a node is a landmark, we add an asterisk.
        """
        if node not in gc.landmarks:
            return str(node)

        # landmark node:
        return "*" + str(node) + "*"

    # Print table lines:
    for node,times in cnt_visited_nodes.most_common(num_most_common):
        ln = " {node:15s} | {times:15d} ".format(\
                node=node_str(node),times=times)
        print(ln)

if __name__ == "__main__":
    measure_load()
