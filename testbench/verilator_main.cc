#include "verilated.h"
#include <iostream>
#include <fstream>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/stat.h>
#include <VTestModule.h>
static uint64_t trace_count = 0;

extern "C" {
    void cluster_statics();
    void cluster_step() {
        while(trace_count%2==1){}
    }
    void mb_step() {
        while(trace_count%8!=0){}
    }
}

double sc_time_stamp()
{
  return trace_count;
}

int main(int argc, char** argv)
{
  unsigned random_seed = (unsigned)time(NULL) ^ (unsigned)getpid();
  uint64_t max_cycles = -1;
  int ret = 0;
  srand(random_seed);
  srand48(random_seed);

  Verilated::randReset(2);
  Verilated::commandArgs(argc, argv);

  VTestModule *top = new VTestModule;
  top->clock = 0;
  while (!Verilated::gotFinish()) {
    if (trace_count%2 == 1) {
        top->clock = 1;
    } else {
        top->clock = 0;
    }
    top->eval();
    trace_count++;
  }

  std::cout << "Done!" << std::endl;
  std::cout << "CPUs statics when finish:" << std::endl;
  std::cout << "--------------------------" << std::endl;
  cluster_statics();
  std::cout << "--------------------------" << std::endl;
  return ret;
}