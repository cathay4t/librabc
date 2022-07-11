# SPDX-License-Identifier: Apache-2.0

import ctypes
from ctypes import (
    c_char_p,
    c_uint32,
    c_uint64,
)
import logging
import json


RABC_PASS = 0

lib = ctypes.cdll.LoadLibrary("librabc.so.0")


class RabcError(Exception):
    def __init__(self, kind, msg):
        self.kind = kind
        self.msg = msg
        super().__init__(f"{kind}: {msg}")


# Opaque struct
class _ClibRabcClient(ctypes.Structure):
    pass


class RabcClient:
    def __init__(self):
        self._c_pointer = ctypes.POINTER(_ClibRabcClient)()
        c_log = c_char_p()
        c_err_msg = c_char_p()
        c_err_kind = c_char_p()
        rc = lib.rabc_client_new(
            ctypes.byref(self._c_pointer),
            ctypes.byref(c_log),
            ctypes.byref(c_err_kind),
            ctypes.byref(c_err_msg),
        )
        process_result(rc, c_log, c_err_kind, c_err_msg)

    def __del__(self):
        if self._c_pointer:
            lib.rabc_client_free(self._c_pointer)

    def poll(self, wait_time):
        if not self._c_pointer:
            raise RabcError("InvalidArgument", "RabcClient not initialied")
        c_log = c_char_p()
        c_err_msg = c_char_p()
        c_err_kind = c_char_p()
        c_events = ctypes.POINTER(c_uint64)()
        event_count = c_uint64(0)
        rc = lib.rabc_client_poll(
            self._c_pointer,
            c_uint32(wait_time),
            ctypes.byref(c_events),
            ctypes.byref(event_count),
            ctypes.byref(c_log),
            ctypes.byref(c_err_kind),
            ctypes.byref(c_err_msg),
        )
        process_result(rc, c_log, c_err_kind, c_err_msg)
        events = list(
            (c_uint64 * event_count.value).from_address(
                ctypes.addressof(c_events.contents)
            )
        )
        lib.rabc_events_free(c_events, event_count)
        return events

    def process(self, event):
        if not self._c_pointer:
            raise RabcError("InvalidArgument", "RabcClient not initialied")
        c_log = c_char_p()
        c_reply = c_char_p()
        c_err_msg = c_char_p()
        c_err_kind = c_char_p()
        rc = lib.rabc_client_process(
            self._c_pointer,
            c_uint64(event),
            ctypes.byref(c_reply),
            ctypes.byref(c_log),
            ctypes.byref(c_err_kind),
            ctypes.byref(c_err_msg),
        )
        process_result(rc, c_log, c_err_kind, c_err_msg)
        if c_reply:
            reply = c_reply.value.decode("utf-8")
            lib.rabc_cstring_free(c_reply)
            return reply
        return None


def parse_log(logs):
    if logs is None:
        return

    log_entries = []
    try:
        log_entries = json.loads(logs.decode("utf-8"))
    except Exception:
        pass
    for log_entry in log_entries:
        msg = f"{log_entry['time']}:{log_entry['file']}: {log_entry['msg']}"
        level = log_entry["level"]

        if level == "ERROR":
            logging.error(msg)
        elif level == "WARN":
            logging.warning(msg)
        elif level == "INFO":
            logging.info(msg)
        else:
            logging.debug(msg)


def process_result(rc, c_log, c_err_kind, c_err_msg):
    err_msg = c_err_msg.value
    err_kind = c_err_kind.value
    parse_log(c_log.value)
    lib.rabc_cstring_free(c_log)
    lib.rabc_cstring_free(c_err_kind)
    lib.rabc_cstring_free(c_err_msg)
    if rc != RABC_PASS:
        raise RabcError(err_kind, err_msg)
