# 12.8.2015
# By real
# A program that calculates the probability of a subset to be "bad", given a
# certain distribution of bad and good nodes.

# The probability for a node to be bad is alpha. For a subset to be bad, at
# least beta of the nodes in it should be bad.

import math


def calc_bad_prob(r,alpha,beta):
    """
    r - Size of a subset.
    alpha - Probability of a node to be bad.
    beta - Amount of bad nodes in a subset to turn it into a bad subset.

    Calculate upper bound for Pr[X > beta*r] using the Chernoff bound.
    This calculation assumes infinite sampling population.
    """
    delta = (beta - alpha) / alpha
    miu = alpha * r # miu = E[x]

    bound = (math.exp(delta) / ((1 + delta)**(1 + delta))) ** miu

    return bound

    










