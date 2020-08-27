# cruise
rusty container manager

## Running it
This project uses runc so it needs to be run on a Linux machine, which I do in a Vagrant box. If you need to, install [Vagrant](https://www.vagrantup.com/). My simple configuration is a `Vagrantfile` with the following contents:
```
Vagrant.configure("2") do |config|
  config.vm.box = "hashicorp/bionic64"
end
```

Then, to setup your environment:
```bash
# in the directory with your Vagrantfile, setup Vagrant box
vagrant up

# login
vagrant ssh

# install gcc
sudo apt-get update
sudo apt install -y gcc

# install rust, this takes a moment
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# add cargo to your PATH environment variable
source $HOME/.cargo/env

# install docker (used to create container root filesystem)
sudo curl -sSL https://get.docker.com/ | sh

# add the vagrant user to the docker group so we don't need to run docker commands as root
sudo usermod -aG docker vagrant

# logout and back in for the group change to take effect
logout
vagrant ssh
```

Now to run the container manager daemon in the Vagrant box:
```bash
# clone the project
git clone https://github.com/willdeuschle/cruise
cd cruise

# build the project (daemon and client)
cargo build

# start the daemon, specifying its root directory and the path to runc
target/debug/daemon run --lib_root=./tmp/lib_root --runtime_path=/usr/bin/runc
```

Now let's interact with the running daemon. In a new shell:
```bash
# in the directory with your Vagrantfile, login to your Vagrant box
vagrant ssh

# create rootfs for container
cd cruise && mkdir -p tmp/rootfs
docker export $(docker create busybox) | tar -C tmp/rootfs -xf -

# create container
target/debug/client container create my_container --rootfs=tmp/rootfs/ sh -- -c "echo hi; sleep 60; echo bye"

# the last command should output: "created: CONTAINER_ID". let's start CONTAINER_ID
target/debug/client container start CONTAINER_ID
```

At this point, if we switch back to the daemon shell, we should see `hi` output over there. If we switch back to our client shell, we can interact some more with our container.
```bash
# get container status
target/debug/client container get CONTAINER_ID
```

For the next minute, we will find that our container is in a `Running` state. After a minute, in our daemon shell, we will see `bye`, and our container will transition into a `Stopped` state. We can now clean up the container:
```bash
# delete container
target/debug/client container delete CONTAINER_ID
```

And if we list our containers, we will see our container no longer exists:
```bash
# list containers
target/debug/client container list
```
