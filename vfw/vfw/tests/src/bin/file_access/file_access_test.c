#include <stdint.h>
#include <string.h>
#include <terminus_cosim.h>
void c_file_access_test()
{
    FD file = fopen("test", FILE_READ | FILE_WRITE | FILE_APPEND);
    char str[] = "c_file_access_test!\n";
    fwrite(file, str, strlen(str));
    fseek(file, 0);
    char buf[256] = {0};
    fread(file, buf, sizeof(buf));
    printf("%s\n", buf);
    fclose(file);
}