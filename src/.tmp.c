#include<stdlib.h>
#include<string.h>
#include<stdio.h>
typedef long long l;int main(int argc,char**argv){l*s=malloc(1024*sizeof(l)),*o=malloc(1024*sizeof(l));size_t p=argc-1,d=0;size_t c=1024,v=1024;for(int i=1;i<argc;i++)s[i-1]=atoll(argv[i]);if(p+1>c){c*=2;s=realloc(s,c*sizeof(l));}l t0_0=(1);s[p+0]=t0_0;p+=1;for(size_t i=p-1;i!=-1;i--)printf("%lld\n", s[i]);}