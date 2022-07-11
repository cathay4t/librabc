import sys
import logging

from rabc import RabcClient


MAX_REPLY = 5
POLL_TIMEOUT = 5


def main():
    init_logger()
    client = RabcClient()
    reply_count = 0
    while reply_count < MAX_REPLY:
        for event in client.poll(POLL_TIMEOUT):
            reply = client.process(event)
            if reply:
                reply_count += 1
                logging.info(f"Got reply from rabcd '{reply}'")


def init_logger():
    root = logging.getLogger()
    root.setLevel(logging.DEBUG)

main()
