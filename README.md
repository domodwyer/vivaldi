## vivaldi

An implementation of the [Vivaldi algorithm] for efficient RTT latency
estimations using an n-dimensional Euclidean model.

<br />
<p align="center">
	<img src="https://iab-assets.s3-eu-west-1.amazonaws.com/vivaldi.gif">
</p>
<br />

* Fully decentralised
* Accurately predicts RTT between any two nodes in the model (± ~11%)
* Low overhead - `O(n)` memory usage for `n^2` network paths
* Quickly adapts to changes in latency between nodes due to re-routing, etc

## The Algorithm

The [Vivaldi algorithm] was presented by Frank Dabek, Russ Cox, Frans Kaashoek &
Robert Morris in 2004, with subsequent improvements suggested in [follow-up]
[papers] by others. 

Most people think the internet is made up of a series of tubes. Indeed it seems
they are wrong - **it's made of springs**.

The authors describe their development of an algorithm to map communication
latencies between nodes to coordinates in N-dimensional Euclidean space, with
the distance between two points roughly equating to the RTT between two nodes.

It models the RTT as the natural length of a physical spring respecting [Hooke's
Law] (which I thought was pretty cool) with the potential energy of each spring
acting as an analogue of estimation error. Minimising the potential energy of
the springs in the system minimises the estimation error of the model. A number
of improvements are made to improve convergence time and accuracy as described
in the paper. It's a good one, you should read it!

## Use It

This implementation is for the algorithm described in the original paper -
there's no update filters (which requires storing more state per node) or
gravity as proposed in the follow-up texts. The caller is responsible for
storing the last known coordinate of each node to later derive RTT estimations
for any pair of nodes.

The Vivaldi algorithm typically piggybacks on top of your normal application
network messages with the coordinates being embedded in request/response
metadata. The application measures the RTT of the request and uses it along with
the coordinate sent by the remote to update the local [`Model`].

## Dimensionality

Although this implementation is generic over any number of dimensions, principle
component analysis by the authors shows there is little benefit beyond 3
dimensions, with 2 dimensions being adequate if overhead is to be kept to a
minimum.

[follow-up]:
https://www.usenix.org/legacy/events/nsdi07/tech/full_papers/ledlie/ledlie_html/index_save.html
[papers]:
https://domino.research.ibm.com/library/cyberdig.nsf/papers/492D147FCCEA752C8525768F00535D8A
[Hooke's Law]: https://en.wikipedia.org/wiki/Hooke%27s_law
[Vivaldi algorithm]: https://pdos.csail.mit.edu/papers/vivaldi:sigcomm/paper.pdf
[`Model`]: (crate::model::Model)