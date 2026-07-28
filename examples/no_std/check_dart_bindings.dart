/// Basic Dart bindings check.
///
/// To run, use the following:
///   dart run check_dart_bindings.dart

import 'dart_no_std_bindings/lib/dart_no_std_bindings.dart';

void main() {
  print("Checking Dart bindings");

  const metadata = A1Meta(
    name: 'foo',
    version: 42,
    payload: [0xde, 0xad, 0xc0, 0xde],
  );
  const value = Protocol(packet: Packet_A1(metadata));

  final result = deserialize<Protocol>(serialize(value));
  final packet = result.packet as Packet_A1;

  assert(packet.item0.name == metadata.name);
  assert(packet.item0.version == metadata.version);
  assert(_listsEqual(packet.item0.payload, metadata.payload));

  print("Done!");
}

bool _listsEqual(List<int> a, List<int> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}
