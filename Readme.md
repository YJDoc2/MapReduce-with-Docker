# MapReduce using Docker compose

This is a proof-of-concept implementation which proves-the-concept of following :

- Imitating a distributed multi-node system using containers and container-orchestrator
- Implementing an ideal case version of Map-Reduce Algorithm, for showing that this, in fact, can emulate distributed system
- Thus using a container-engine and container-orchestrator one can try-out distributed algorithms and get their hands in distributed programming

Currently this is a bit of mess, but is capable of getting converted into a full-fledged library implementation, If anyone is interested, please open an issue :slightly_smiling_face: :slightly_smiling_face:

The file books.txt contains books from project Gutenberg : Pride and prejudice, Alice's adventures in wonderland, The adventures of sherlock holmes as data for testing.

## Motivation

This started as for a Big Data course I was taking, we had to install hadoop on our local system to implement and run map-reduce program as part of lab work. For some reason I could not get the hadoop working on my system, and I wanted an easier way to write and run such distributed programs.

Of course this is by no means meant to be some "replacement" or even imitation of hadoop, but more of a limited emulation of it, sing some container engine (Docker) and Rust (Which I feel are much more easier to install, personally) someone can start with writing and testing programs for distributed systems much easily.

## Points to Note

As said before this implements an idea case version of Map-Reduce algorithm to prove that the distributed system actually works as expected : The original paper can be found at https://static.googleusercontent.com/media/research.google.com/en//archive/mapreduce-osdi04.pdf

The paper describes algorithm which takes in consideration things such as network latency, an underlying distributed filesystem, occasional failure of slave nodes, etc. The ideal version implemented here assumes that :

- slave nodes are always up, and they are up before master starts
- slave nodes are always available with no failures, and will always complete the work they are assigned
- the underlying files system is imitated using docker volumes, where the same directory in host system is mapped into each of the containers, giving an impression to nodes that they have a shared filesystem.

## An Use-oriented Overview

This sections gives a short overall explanation of the project and its working from using perspective. More detailed internal working is explained later.

This repository actually consists of two Rust projects :

- a distributed implementation program which shows word-count using map-reduce using master and N slave nodes
- a single node program, which does word counting on a single node

The singlenode directory contains a complete rust application which does the single-node word counting, and the root directory itself is for the distributed application.

The Dockerfile in root is for the distributed version, as each node in the system is built using this dockerfile. This also contains docker-compose file which specifies the configuration of the system, including number of nodes, mapping of folders and env-vars indicating which nodes will act as slave and which will act as master node.

To emulate the single node and multi-node node system there are few knobs that can be adjusted :

- number of nodes : This is the simplest one, where adding more nodes in docker-compose file as slaves and connecting them to the first (master) node will increase the performance to a certain limit,after which the network latency will have more overhead. The curve of nodes v/s overhead will roughly take the shape of 'U' where lower number will make number of parallelly running units less, and thus along with the overhead of splitting the data, connecting and communicating with the nodes will create a higher overhead, and thus single node system might perform better. A medium number of nodes will have the most benefits, and a much higher number of node will again introduce large overhead of communicating with them, and splitting files into small parts.
- cpu_quota in docker-compose : cpu_quota field determines maximum amount of cpu time share given to a particular container. See https://docs.docker.com/config/containers/resource_constraints/ for more information. This artificially chokes the processing powers for reasons explained later. For comparing single node and multi node, make sure that cpu_quota in both docker-compose files is set to the same value.
- Input File Size : To test the efficiency of distributed system a large sized file needs to be used, as on small scale, the communication and data splitting in distributed configuration introduce more overhead, and single node system may outperform. The data folder contains three files :
  - test.txt : contains `So, This is a long file....` line
    times.
  - bigtext.txt : contains 100 lines of lorem-ipsum text, each of 500(?\*) words.
    ( \* ) : I don't remember how many words are there per line at the time of writing this, but they seem approximately 500. To know exact, run map-reduce with this file input :wink: :wink:
  - books.txt : is the largest file, and contains text form Pride and prejudice, Alice's adventures in wonderland, The adventures of sherlock holmes books, taken from project Gutenberg.

## Flow diagram
