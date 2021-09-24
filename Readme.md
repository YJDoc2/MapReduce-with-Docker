# MapReduce using Docker compose

This is a proof-of-concept implementation which proves-the-concept of following :

- Imitating a distributed multi-node system using containers and container-orchestrator
- Implementing an ideal case version of Map-Reduce Algorithm, for showing that this, in fact, can emulate distributed system
- Thus using a container-engine and container-orchestrator one can try out distributed algorithms and get their hands in distributed programming

Currently this is a bit of mess, but is capable of getting converted into a full-fledged library implementation, If anyone is interested, please open an issue :slightly_smiling_face: :slightly_smiling_face:

The file books.txt contains books from project Gutenberg : Pride and prejudice, Alice's adventures in wonderland, The adventures of sherlock holmes as data for testing.

## Motivation

This started, as for a Big Data course I was taking, we had to install hadoop on our local system to implement and run map-reduce program as part of lab work. For some reason I could not get the hadoop working on my system, and I wanted an easier way to write and run such distributed programs.

Of course this is by no means meant to be some "replacement" or even imitation of hadoop, but more of a limited emulation of it, using some container engine (Docker) and Rust (Which I feel are much more easier to install, personally) someone can start with writing and testing programs for distributed systems much easily.

I also wanted to explore the async functionality in Rust, so this uses Tokio to run the listeners on TCP for master and worker node, _that was fun_.

## Points to Note

As said before this implements an ideal case version of Map-Reduce algorithm to prove that the distributed system actually works as expected : The original paper can be found at https://static.googleusercontent.com/media/research.google.com/en//archive/mapreduce-osdi04.pdf

The paper describes algorithm which takes in consideration things such as network latency, an underlying distributed filesystem, occasional failure of worker nodes, etc. The ideal version implemented here assumes that :

- worker nodes are always up, and they are up before master starts
- worker nodes are always available with no failures, and will always complete the work they are assigned
- the underlying files system is imitated using docker volumes, where the same directory in host system is mapped into each of the containers, giving an impression to nodes that they have a shared filesystem.

## An Use-Oriented Overview

This sections gives a short overall explanation of the project and its working from using perspective. More detailed internal working is explained later.

To start the single node system : go in the `singlenode` directory and run

```
docker-compose up
```

This will start the docker container and run the single node word-count program. The output will be generated in `data` directory in the project root with name singlenode.txt . The output on console will show time taken for the program execution, measured from `main` function's start to end.

To start the multi-node system system run the following in the project root

```
docker-compose up
```

This will start the worker containodesner first, and then start master node and attach them to each other. The brief output on console will show progress and queuing of various tasks, and in the end, it will show the time taken for the process, measured from start of the `mater_main` function to end of `master_main` function. Then the master node will exit, and rest of containers will need to be stopped manually by using `ctrl+c`, which will stop the docker compose.

The output will be generated in the project root data directory, with following format:<br />
map_split\_\*.txt will contain output of map stage<br />
shuffle_split\_\*.txt will contain output of shuffle stage<br />
reduce_split\_\*.txt will contain result of reduce stage<br />

map_split files will be equal to number of worker nodes, but the number of shuffle and reduce splits files will depend on the keys generated in map stage, and hash functions used to hash the keys.

This repository actually consists of two Rust projects :

- a distributed implementation program which shows word-count using map-reduce using master and N worker nodes
- a single node program, which does word counting on a single node

The singlenode directory contains a complete rust application which does the single-node word counting, and the root directory itself is for the distributed application.

The Dockerfile in root is for the distributed version, as each node in the system is built using this dockerfile. This also contains docker-compose file which specifies the configuration of the system, including number of nodes, mapping of directories and env-vars indicating which nodes will act as worker and which will act as master node.

To emulate the single node and multi-node node system there are few knobs that can be adjusted :

- number of nodes : This is the simplest one, where adding more nodes in docker-compose file as workers and connecting them to the first (master) node will increase the performance to a certain limit,after which the network latency will have more overhead. The curve of nodes v/s overhead will roughly take the shape of 'U' where lower number will make number of parallelly running units less, and thus along with the overhead of splitting the data, connecting and communicating with the nodes will create a higher overhead, and thus single node system might perform better. A medium number of nodes will have the most benefits, and a much higher number of node will again introduce large overhead of communicating with them, and splitting files into small parts.
  <br />
- cpu_quota in docker-compose : cpu_quota field determines maximum amount of cpu time share given to a particular container. See https://docs.docker.com/config/containers/resource_constraints/ for more information. This artificially chokes the processing powers for reasons explained later. For comparing single node and multi node, make sure that cpu_quota in both docker-compose files is set to the same value.
  <br />
- Input File Size : To test the efficiency of distributed system a large sized file needs to be used, as on small scale, the communication and data splitting in distributed configuration introduce more overhead, and single node system may outperform. The data directory contains three files :
  - test.txt : contains `So, This is a long file....` line 60 times.
  - bigtext.txt : contains 100 lines of lorem-ipsum text, each of 500(?\*) words.
    ( \* ) : I don't remember how many words are there per line at the time of writing this, but they seem approximately 500. To know exact, run word count with this file input :wink: :wink:
  - books.txt : is the largest file, and contains text form Pride and prejudice, Alice's adventures in wonderland, The adventures of sherlock holmes books, taken from project Gutenberg.

Now to see how the distributed v/s single node performs, we can lower the cpu_quota, and see which takes more time, as if allowed 100%, both of them will execute with 100% access to cpu, and there may not be much visible difference. Here, the cpu_quota acts like a zoom-in knob, with lesser cpu_quota amplifying the difference between single node and multi-node system.
Another way to see this would be to perform operation on large file, as distributed will split file, and (hopefully) run the processing parallely (this depends on number of cores and other factors also), whereas single node will process the file linearly. Thus larger file size should also amplify the difference between two systems. Getting the file size large enough to make the difference considerable on decently configured(with multiple core cpu) system is difficult : the file size required might go in size of GiBs or more. Thus a combination of slightly larger files, with smaller cpu_quota would be effective in showing the difference well.

The compose files must have the following env vars defined :

- TYPE : type of node, can be one of `master` or `worker`, and there must be only one master. This must be defined for each of the nodes
- INPUT : path of data file that is to be processed, inside the container. This is to be defined only for master node.

The Dockerfile for the distributed system defines the image configuration, where it exposes two of the ports, and installs `nmap`, which is used for port-scanning and worker node detection by master.

## Flow diagram

This section should contain the flow diagram for the communication between the nodes, and how the processing in distributed system takes place. _(But it doesn't)_

## Developer Overview

This section explains the file structure and what each file contains.

```sh
.
├── data                      -> Contains text files for tests. Also the output of
|                                 single and distributed system will be generated here
├── manager                   -> Contains lib for master-worker manager code
|     ├──src
|     |   ├── broker.rs       -> Contains functions for spawning the listener threads for the manager
|     |   ├── manger.rs       -> Manager struct, which maintains worker pool, maintains
|     |                           the message queue, and takes care of queuing the work to workers
|     |   └── messages.rs     -> Contains Messages enum for master-worker communication
|     └── docker-compose.yaml -> Compose file for running the single node program
├── src
|   ├── master
|   |   └── master.rs         -> Contains functionality of master node, such as splitting the file,
|   |                             queuing all the work, and main function for the master
|   ├── slave
|   |   ├── hasher.rs         -> Contains a simple hashing function for strings,
|   |                             which is used in shuffling step
|   |   ├── map.rs            -> Contains map function of the map step
|   |   ├── reduce.rs         -> Contains reduce function of the reduce step
|   |   ├── shuffle.rs        -> Contains shuffle function of the shuffle step
|   |   └── slave.rs          -> Contains functionality of worker nodes,
|   |                             and main function for the worker
|   ├── ip_finder.rs          -> Contains functions to get node's
|   |                             own ip and scan ports to find worker nodes
|   ├── main.rs               -> Contains main function of the complete multi-node system binary
|   └── mapreduce.rs          -> This is a relic from initial thoughts, contains two traits which
|                                 were meant to be used for mapper and reducer struct,
|                                 but are not actually used anywhere.
├── docker-compose.yaml       -> Compose file for running the multi-node system
├── Dockerfile                -> Contains Docker image specification for the multi-node system image
├── Readme.md                 -> This file
└── run.sh                    -> To get the multi-node system running quickly

```

## License

This code is released under GNU GPL V3, see [License](https://github.com/YJDoc2/MapReduce-with-Docker/blob/main/License) file for more info.
