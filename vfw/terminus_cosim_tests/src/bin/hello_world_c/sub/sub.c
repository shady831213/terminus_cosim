#include "sub.h"
void task1()
{
    printf("This is task1\n");
}

void task2(uint32_t parent, void *subtask, uint32_t *subargs_ptr)
{
    printf("subargs %d\n", *subargs_ptr);
    uint32_t task_id = fork(subtask, *subargs_ptr);
    printf("This is task2! parent:%d\n", parent);
    join(task_id);
    printf("join task3 in task2\n");
}

void task3(uint32_t parent)
{
    for (int i = 0; i < 10; i++)
    {
        printf("This is task3! parent:%d, %d\n", parent, i);
    }
}