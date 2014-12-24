"""
22.12.2014
by real
http://www.freedomlayer.org

We route messages in the network by random walking from source to destination.
We use the information from the network coordinates to make the random walk a
bit smarter.

This code checks the performance of routing messages using this method. We
simulate delivery of many messages, and we find the average amount of hops it
takes for a message to reach its destination.

In addition, we also check the uniquness of network coordinates. We count the
occurences of unique coordinates, and print back the coordinate that occured
the maximum amount of time. (We hope that this maximum value is 1).
"""

import graph_coords


def check_random_walk():
    # Amount of messages to simulate:
    num_messages = 0x20
    # Sizes of graphs to check. The resulting graph will have size of about
    # (2**i):
    i_range = range(6,16)
    # Function used to generate the graph:
    gen_graph_func = graph_coords.gen_gnp_graph
    # Base used to choosing weights for neighbours in random walk:
    base=1

    print("Naive random walking")
    print("---------------------------")

    print("||| graph generation func =",gen_graph_func.__name__)
    print("||| i's range =",i_range)
    print("||| num_messages =",num_messages)
    print("||| base = ",base)
    print()

    # Print table's header:
    header_ln = (" {i:3s} | {k:6s} | {avg_num_hops:15s} | {max_node_visits:16s} | "
                 "{max_coord_occur:16s} ").format(\
            i="i",k="k",avg_num_hops="Avg num hops"\
            ,max_node_visits="Max Node Visits",max_coord_occur="Max Coord Occur")

    print(header_ln)
    print('-' * len(header_ln))

    for i in range(6,16):
        k = i**2
        # Generate graph:
        g = gen_graph_func(i)
        # Generate coordinates:
        gc = graph_coords.GraphCoords(graph=g,k=k)
        # Simulate Delivery of num_messages messages:
        avg_num_hops,cnt_visited_nodes =\
                gc.get_avg_num_hops(num_messages,base=base)

        # Extract the most visited node with respect to random walks:
        max_node,max_node_visits = cnt_visited_nodes.most_common(1)[0]

        # Count coordinates:
        cnt_coord = gc.count_coords()

        # Extract the most common coordinate:
        max_coord,max_coord_occur = cnt_coord.most_common(1)[0]


        table_ln = (" {i:3d} | {k:6d} | {avg_num_hops:15f} | {max_node_visits:16d} |"
                    "{max_coord_occur:16d} ").format(\
                i=i,k=k,avg_num_hops=avg_num_hops,\
                max_node_visits=max_node_visits,max_coord_occur=max_coord_occur)
        print(table_ln)


if __name__ == "__main__":
    check_random_walk()
