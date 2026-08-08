/* count-interpose.c — S-119 C-lite gamba Zend (deroga 3 del criterio):
 * conta gli EVENTI malloc/calloc/realloc/free del processo oracle con
 * USE_ZEND_ALLOC=0 (ogni emalloc diventa un malloc: il NUMERO di eventi e'
 * cio' che si misura, il tempo no). Interposizione DYLD classica; il dump va
 * su file (env CLITE_COUNT_OUT, append) a atexit — mai su stdout/stderr, che
 * il giudice legge. realloc disaggregato, come la convenzione phpr
 * (A-LE-104-1). Compilare: clang -O2 -dynamiclib -o count-interpose.dylib
 * count-interpose.c
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdatomic.h>

static _Atomic unsigned long long n_malloc, n_calloc, n_realloc, n_free;
/* PHP rimpiazza environ durante l'avvio: a fine processo getenv() torna NULL
 * (misurato, S-119). Il path si cattura nel COSTRUTTORE, prima che il SAPI
 * tocchi l'ambiente. */
static char out_path[1024];
__attribute__((constructor)) static void grab_env(void) {
    const char *p = getenv("CLITE_COUNT_OUT");
    if (p && *p) { strncpy(out_path, p, sizeof out_path - 1); }
}
/* Distruttore d'immagine, non atexit: PHP puo' uscire per una via che salta
 * gli handler atexit registrati pigramente da una dylib inserita; il
 * destructor di dyld corre comunque allo smontaggio del processo. */
__attribute__((destructor)) static void dump(void) {
    if (!out_path[0]) return;
    FILE *f = fopen(out_path, "a");
    if (!f) return;
    fprintf(f, "countinterpose pid=%d malloc_n=%llu calloc_n=%llu realloc_n=%llu free_n=%llu\n",
            (int)getpid(), n_malloc, n_calloc, n_realloc, n_free);
    fclose(f);
}
void *my_malloc(size_t s) { n_malloc++; return malloc(s); }
void *my_calloc(size_t n, size_t s) { n_calloc++; return calloc(n, s); }
void *my_realloc(void *q, size_t s) { n_realloc++; return realloc(q, s); }
void my_free(void *q) { n_free++; free(q); }

typedef struct { const void *replacement, *replacee; } interpose_t;
__attribute__((used, section("__DATA,__interpose"))) static const interpose_t tab[] = {
    { (const void *)my_malloc,  (const void *)malloc  },
    { (const void *)my_calloc,  (const void *)calloc  },
    { (const void *)my_realloc, (const void *)realloc },
    { (const void *)my_free,    (const void *)free    },
};
