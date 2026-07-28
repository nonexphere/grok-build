# Realtime Voice — SPECS

Status: backlog documentation only. Future full-duplex voice consumes Session,
Turn/Item lifecycle, Interaction, backpressure and transport security contracts.
This pass adds no voice protocol schema, crate or runtime implementation.
[D-BK.1,D-BK.2]

Futuro client/adapter usa Session stream, Turn input e interrupt first-class.
Evolui `xai-grok-voice`; não redefine App Server, MCP ou Tower.

Gate futuro: latency, VAD, partial STT, streaming TTS, barge-in, privacy e
hardware failure behavior.
