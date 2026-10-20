/* Native, file-controlled CLI fixture: no interpreter or provider requests. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <limits.h>
#include <sys/stat.h>
extern char **environ;
static char base[PATH_MAX];
static FILE *file(const char *name, const char *mode) {
    char path[PATH_MAX];
    snprintf(path, sizeof path, "%s/%s", base, name);
    return fopen(path, mode);
}
static int exists(const char *name) { FILE *f=file(name,"r"); if(!f)return 0; fclose(f); return 1; }
static int status(const char *name) { int n=0; FILE *f=file(name,"r"); if(f){fscanf(f,"%d",&n);fclose(f);} return n; }
static void output(const char *name) { FILE *f=file(name,"r"); if(!f)exit(94); int c;while((c=fgetc(f))!=EOF)putchar(c);fclose(f); }
static int eq(int argc,char **argv,int n,const char **want) {
    if(argc!=n+1)return 0;
    for(int i=0;i<n;i++)if(strcmp(argv[i+1],want[i]))return 0;
    return 1;
}
int main(int argc,char **argv) {
    snprintf(base,sizeof base,"%s",argv[0]); char *slash=strrchr(base,'/'); if(!slash)return 93; *slash=0;
    if(exists("pause")){fclose(file("entered","w"));while(!exists("release"))usleep(10000);}
    int delay=status("delay-ms"); if(delay>0)usleep((unsigned int)delay*1000);
    FILE *env=file("environment","w"); for(char **e=environ;*e;e++)fprintf(env,"%s\n",*e);fclose(env);
    FILE *calls=file("calls","a"); for(int i=1;i<argc;i++)fprintf(calls,"%s%s",i>1?" ":"",argv[i]);fprintf(calls,"\n");fclose(calls);
    if(eq(argc,argv,1,(const char*[]){"--version"})){output("version");return 0;}
    if(eq(argc,argv,1,(const char*[]){"--help"}) || eq(argc,argv,2,(const char*[]){"exec","--help"})){output("help");return 0;}
    if(eq(argc,argv,3,(const char*[]){"--setting-sources","bogus","--help"})){
        if(exists("help-before-validation")){output("help");return 0;}
        fputs("error: invalid setting sources\n",stderr);return 1;
    }
    if(eq(argc,argv,3,(const char*[]){"--setting-sources","","--help"})) {
        output("help");if(exists("settings-error"))fputs("error: invalid empty settings\n",stderr);
        return status("settings-status");
    }
    if(eq(argc,argv,2,(const char*[]){"login","status"}) || eq(argc,argv,4,(const char*[]){"--setting-sources","","auth","status"})) {
        if(exists("mutate")){
            FILE *old=fopen(argv[0],"rb"), *next=file("replacement","wb");
            int c;while((c=fgetc(old))!=EOF)fputc(c,next);fputs("changed",next);fclose(old);fclose(next);
            char replacement[PATH_MAX];snprintf(replacement,sizeof replacement,"%s/replacement",base);
            if(chmod(replacement,0700))return 99;
            if(rename(replacement,argv[0]))return 98;
        }
        return status("login");
    }
    /* The expected vector is NUL delimited, preserving empty arguments. */
    FILE *expected=file("expected-argv","rb");
    if(!expected)return 95;
    for(int i=1;i<argc;i++) {char value[8192];size_t n=0;int c;
        while((c=fgetc(expected))!=EOF && c!=0 && n+1<sizeof value)value[n++]=(char)c;
        value[n]=0;if(c!=0 || strcmp(value,argv[i])){fclose(expected);return 96;}}
    if(fgetc(expected)!=EOF){fclose(expected);return 97;}fclose(expected);
    fclose(file("exec-entered","w"));
    char cwd[PATH_MAX]; if(!getcwd(cwd,sizeof cwd))return 103;
    FILE *cwd_file=file("cwd","w");fputs(cwd,cwd_file);fclose(cwd_file);
    if(exists("exec-pre-output")){output("exec-pre-output");fflush(stdout);}
    if(exists("exec-stderr")){
        FILE *err=file("exec-stderr","r");int ch;
        while((ch=fgetc(err))!=EOF)fputc(ch,stderr);fclose(err);fflush(stderr);
    }
    if(exists("exec-pause")){while(!exists("exec-release"))usleep(10000);}
    FILE *in=file("stdin","w");int c;while((c=getchar())!=EOF)fputc(c,in);fclose(in);
    fclose(file("stdin-eof","w"));
    if(exists("exec-source")){
        if(mkdir("src",0700) && access("src",F_OK))return 100;
        FILE *source=file("exec-source","r"), *out=fopen("src/output.rs","w");
        if(!source || !out)return 101;
        int ch;while((ch=fgetc(source))!=EOF)fputc(ch,out);fclose(source);fclose(out);
    }
    if(exists("exec-last")){
        for(int i=1;i+1<argc;i++)if(!strcmp(argv[i],"-o")){
            FILE *source=file("exec-last","r"), *out=fopen(argv[i+1],"w");
            if(!source || !out)return 102;
            int ch;while((ch=fgetc(source))!=EOF)fputc(ch,out);fclose(source);fclose(out);
        }
    }
    output("exec-output");return status("exec-status");
}
