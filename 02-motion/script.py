import datetime
import platform
import queue
import re
import socket
import sys
import threading
import time

import matplotlib.pyplot as plt

tello_address = ('192.168.10.1', 8889)

def main():
    sock = connect_to_tello('', 9000)
    q = queue.Queue()
    telemetry = TelemetryThread(q, connect_to_tello('0.0.0.0',8890, False))
    telemetry.start()
    send_command(sock, "takeoff")
    send_command(sock, "left 20")
    send_command(sock, "right 20")
    send_command(sock, "speed 10")
    send_command(sock, "cw 360")
    send_command(sock, "flip l")
    send_command(sock, "land")
    q.put("STOP")
    print(send_command(sock, "battery?"))
    sock.close()
    plot_pry(telemetry.get_pitch(), telemetry.get_roll(), telemetry.get_yaw())


def plot_pry(pitch, roll, yaw):
    fig, ax = plt.subplots(1, figsize=(8,6))
    fig.suptitle('pitch roll yaw')
    x = range(len(pitch))

    ax.plot(x, pitch, color="red", label="pitch")
    ax.plot(x, roll, color="green", label="roll")
    ax.plot(x, yaw, color="blue", label="yaw")
    plt.legend(loc="lower right", title="Legend Title", frameon=False)
    plt.show()

def connect_to_tello(local_ip, local_port, start_sdk = True):
    tello_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    tello_socket.bind((local_ip, local_port))
    if start_sdk :
        send_command(tello_socket, "command")
    return tello_socket

def send_command(socket, message):
    socket.sendto(message.encode(encoding="utf-8"), tello_address)
    data, server = socket.recvfrom(2048)
    print(f"{server}, {data}")
    return data.decode()


class TelemetryThread(threading.Thread):
    def __init__(self, queue, listening_socket, args=(), kwargs=None):
        threading.Thread.__init__(self, args=args, kwargs=kwargs)
        self.queue = queue
        self.telemetry = {"pitch":[], "roll":[], "yaw":[]}
        self.regex = re.compile(r"^pitch:(?P<pitch_val>-?\d+?);roll:(?P<roll_val>-?\d+?);yaw:(?P<yaw_val>-?\d+?);.*$")
        self.run_loop = True
        self.listening_socket = listening_socket
        

    def run(self):
        while self.run_loop:
            data = self.listening_socket.recvfrom(2048)
            msg = self.regex.match( data[0].decode())
            grp = msg.group('pitch_val', 'roll_val', 'yaw_val')
            self.telemetry["pitch"].append(int(grp[0]))
            self.telemetry["roll"].append(int(grp[1]))
            self.telemetry["yaw"].append(int(grp[2]))
            print(f"{grp}")
            if not self.queue.empty():
                val = self.queue.get()
                print(val)
                if val == "STOP":
                    self.run_loop = False
        self.listening_socket.close()

    def get_pitch(self):
        return self.telemetry["pitch"]
    
    def get_roll(self):
        return self.telemetry["roll"]

    def get_yaw(self):
        return self.telemetry["yaw"]


if __name__ == "__main__":
    main()
