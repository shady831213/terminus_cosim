#ifndef __SUB_H__
#define __SUB_H__
#include <stdint.h>
#include <terminus_cosim.h>
void task1();

void task2(uint32_t parent, void *subtask, uint32_t *subargs_ptr);

void task3(uint32_t parent);
#endif