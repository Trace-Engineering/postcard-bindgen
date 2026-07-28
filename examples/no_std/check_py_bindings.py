# Basic Python bindings check.
#
# To run, use the following:
#   py check_py_bindings.py

from py_no_std_bindings.types import *
from py_no_std_bindings.ser import serialize
from py_no_std_bindings.des import deserialize

print("Checking Python bindings")

a1_meta = A1Meta("foo", 42, [0xDE, 0xAD, 0xC0, 0xDE])
bs = serialize(a1_meta)
a1_meta_des, remaining = deserialize(A1Meta, bs)
assert a1_meta == a1_meta_des
assert len(remaining) == 0

packet_a1 = Packet_A1(a1_meta)
bs = serialize(packet_a1)
packet_a1_des, remaining = deserialize(Packet, bs)
assert packet_a1 == packet_a1_des
assert len(remaining) == 0

protocol = Protocol(packet_a1)
bs = serialize(protocol)
protocol_des, remaining = deserialize(Protocol, bs)
assert protocol == protocol_des
assert len(remaining) == 0

print("Done!")
