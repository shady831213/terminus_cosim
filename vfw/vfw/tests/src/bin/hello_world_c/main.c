#include <stdint.h>
#include <terminus_cosim.h>
#include "sub.h"
static uint32_t subargs = 100;

uint32_t main()
{
    uint32_t hart_id = hartid();
    try_fork(task3, hart_id);
    uint32_t task1_id = try_fork(task1);
    uint32_t task2_id = fork_on(1, task2, hart_id, task3, &subargs);

    join(task1_id);
    for (int i = 0; i < 20; i++)
    {
        printf("hello %s, %d, %f, %x, %x, %% \n", "cprint", i, float_to_arg(1.2345), 0x1234, 0x1234);
    }
    join(task2_id);
    exit(8);
    return 0xf;
}