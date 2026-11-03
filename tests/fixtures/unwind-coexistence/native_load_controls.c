// Entire worker stack is C: never cancel or pthread_exit through Rust frames.
#include <execinfo.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <string.h>
#include <sched.h>
static atomic_int ready, cleaned;
static void cleanup(void *p) { (void)p; atomic_fetch_add(&cleaned, 1); }
static void *worker(void *mode) {
    pthread_cleanup_push(cleanup, NULL);
    if (mode) pthread_exit((void *)42);
    atomic_store(&ready, 1);
    for (;;) { pthread_testcancel(); sched_yield(); }
    pthread_cleanup_pop(1);
    return NULL;
}
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    if (!strcmp(argv[1], "backtrace")) {
        void *frames[32]; int count = backtrace(frames, 32);
        if (count <= 0) return 3;
        backtrace_symbols_fd(frames, count, 1); return 0;
    }
    int exiting = !strcmp(argv[1], "pthread-exit");
    if (!exiting && strcmp(argv[1], "cancel")) return 2;
    pthread_t t; void *result;
    if (pthread_create(&t, NULL, worker, exiting ? (void *)1 : NULL)) return 4;
    if (!exiting) {
        while (!atomic_load(&ready)) sched_yield();
        if (pthread_cancel(t)) return 5;
    }
    if (pthread_join(t, &result)) return 6;
    if (atomic_load(&cleaned) != 1) return 7;
    if (result != (exiting ? (void *)42 : PTHREAD_CANCELED)) return 8;
    puts("native-cleanup-ok"); return 0;
}
