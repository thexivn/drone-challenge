import platform
import socket
import sys
import threading
import time

tello_address = ('192.168.10.1', 8889)

def main():
    sock = connect_to_tello(9000)
    print(send_command(sock, "battery?"))
    sock.close()

def connect_to_tello(localPort, start_sdk = True):
    tello_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    tello_socket.bind(('', localPort))
    if start_sdk :
        send_command(tello_socket, "command")
    return tello_socket

def send_command(socket, message):
    socket.sendto(message.encode(encoding="utf-8"), tello_address)
    data, server = socket.recvfrom(2048)
    return data.decode(endcoding="utf-8")

if __name__ == "__main__":
    main()
