**Your computer can double up as a cosmic ray detector. Yes, really!**

[Cosmic rays](https://en.wikipedia.org/wiki/Cosmic_ray) hit your computer all the time. If they hit the RAM, this can [sometimes cause disturbances](https://en.wikipedia.org/wiki/Soft_error#Cosmic_rays_creating_energetic_neutrons_and_protons), like flipping a random bit in memory.
To use your computer as a cosmic ray detector, simply run this program!  
The detection works by allocating a vector of zeroed bytes and then checking regularly to see if they are all still zero. Ta-da!

* Do not run this on a computer with [ECC memory](https://en.wikipedia.org/wiki/ECC_memory), as that will prevent the data corruption we are trying to detect!
* The chance of detection increases with the physical size of your DRAM modules and the percentage of them you allocate to this program.
* Beware of operating systems being clever, and e.g. compressing unused memory pages. A vector of nothing but zeros that hasn't been used in 30 seconds is an excellent target for this. This will shrink your detector!
* Expect detections to be *very* rare.


**Special thanks to**
* /u/csdt0 and /u/HeroicKatora on reddit for ideas about how to improve the correctness of the program and avoid the pitfalls of virtual memory.


## CSV file format
- Program start entry containing the time when program started, Check interval (in ms), Number of checks that failed to find a bitflip, detected type (0 - normal bit flip, 1 - bit flip was detected but can no longer be found), end check interval time

# Usage:
```cargo run --color=always -- -d 5000 -m 1kB --longitude 10.11 --latitude '\-11.12' --file-path ~/CosmicRays/results.txt```