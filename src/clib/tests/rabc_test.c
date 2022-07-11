#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <rabc.h>

#define WAIT_TIME               10
#define PROCESS_LOOP_COUNT      10

int process(struct rabc_client *client) {
    int rc = EXIT_SUCCESS;
    uint32_t ret = RABC_PASS;
    uint64_t *events = NULL;
    uint64_t event_count = 0;
    uint64_t i = 0;
    char *log = NULL;
    char *err_kind = NULL;
    char *err_msg = NULL;
    char *reply = NULL;

    ret = rabc_client_poll(client, WAIT_TIME,
                           &events, &event_count, &log, &err_kind,
                           &err_msg);
    printf("Log %s\n", log);
    rabc_cstring_free(log);

    if (ret != RABC_PASS) {
        printf("Error: %s: %s\n", err_kind, err_msg);
        rc = EXIT_FAILURE;
        rabc_cstring_free(err_kind);
        rabc_cstring_free(err_msg);
        goto out;
    }

    for (i=0; i < event_count; ++i) {
        ret = rabc_client_process(client, events[i], &reply, &log, &err_kind,
                                  &err_msg);
        printf("Log %s\n", log);
        rabc_cstring_free(log);
        if (ret != RABC_PASS) {
            printf("Error: %s: %s\n", err_kind, err_msg);
            rc = EXIT_FAILURE;
            rabc_cstring_free(err_kind);
            rabc_cstring_free(err_msg);
            goto out;
        } else {
            printf("Reply: %s\n", reply);
            rabc_cstring_free(reply);
        }
    }

out:
    rabc_events_free(events, event_count);
    return rc;
}

int main(void) {
    int rc = EXIT_SUCCESS;
    uint32_t ret = RABC_PASS;
    struct rabc_client *client = NULL;
    char *err_kind = NULL;
    char *err_msg = NULL;
    char *log = NULL;
    int i = 0;

    ret = rabc_client_new(&client, &log, &err_kind, &err_msg);
    printf("Log %s\n", log);

    if (ret != RABC_PASS) {
        printf("Error: %s: %s\n", err_kind, err_msg);
        rc = EXIT_FAILURE;
        goto out;
    }

    for (i = 0; i < PROCESS_LOOP_COUNT; ++i) {
        if (process(client) != EXIT_SUCCESS) {
            goto out;
        }
    }

 out:
    rabc_cstring_free(err_kind);
    rabc_cstring_free(err_msg);
    rabc_cstring_free(log);
    rabc_client_free(client);
    exit(rc);
}
