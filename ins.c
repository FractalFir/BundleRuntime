#include <stdio.h>
#include <stdint.h>
void ret(){return;}
void print_fnc(void* fnc, int bytes){
  char* code = fnc;
  for(int i = 0; i < bytes; i++){
    uint8_t byte = (int)(code[i]);
    int b = 0 + byte;
    printf("%o,",b);
  }
   printf("\n--------------------------------------------------------\n");
}
int main(){
     printf("Ret:\n");
     print_fnc(ret,16);
}
