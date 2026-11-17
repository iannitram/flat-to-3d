// rd-trigger: ask a running RenderDoc-injected program to capture a frame, with no key press.
// Built against RenderDoc 1.46's replay API headers; links the host's librenderdoc.so.
// Usage: rd-trigger [--delay SECONDS] [--frames N] [--wait SECONDS]
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <set>
#include <thread>
#include <unistd.h>
#include "renderdoc_replay.h"

// Mark this as a replay program so the library does not make it a capture target itself.
REPLAY_PROGRAM_MARKER()

static void sleep_s(double s) { std::this_thread::sleep_for(std::chrono::duration<double>(s)); }

int main(int argc, char **argv) {
  double delay = 0, wait = 60;
  uint32_t frames = 1;
  for (int i = 1; i < argc; i++) {
    if (!strcmp(argv[i], "--delay") && i + 1 < argc) delay = atof(argv[++i]);
    else if (!strcmp(argv[i], "--frames") && i + 1 < argc) frames = (uint32_t)atoi(argv[++i]);
    else if (!strcmp(argv[i], "--wait") && i + 1 < argc) wait = atof(argv[++i]);
    else { fprintf(stderr, "usage: rd-trigger [--delay SECONDS] [--frames N] [--wait SECONDS]\n"); return 2; }
  }
  if (frames == 0) { fprintf(stderr, "rd-trigger: --frames must be at least 1\n"); return 2; }
  if (delay > 0) { fprintf(stderr, "rd-trigger: capturing in %.0f s\n", delay); sleep_s(delay); }

  GlobalEnvironment env;
  env.enumerateGPUs = false;
  RENDERDOC_InitialiseReplay(env, rdcarray<rdcstr>());

  // Use the highest ident (usually the newest target): the game, not a launcher that may also have the layer loaded.
  // Skip our own process in case the marker is not honoured.
  uint32_t ident = 0, last = 0;
  while ((ident = RENDERDOC_EnumerateRemoteTargets(rdcstr(), ident)) != 0) {
    ITargetControl *probe = RENDERDOC_CreateTargetControl(rdcstr(), ident, rdcstr("rd-trigger"), false);
    if (probe) {
      if (probe->Connected() && probe->GetPID() != (uint32_t)getpid()) last = ident;
      probe->Shutdown();
    }
  }
  if (last == 0) {
    fprintf(stderr, "rd-trigger: no program with RenderDoc loaded is running\n");
    RENDERDOC_ShutdownReplay();
    return 1;
  }
  ITargetControl *tc = RENDERDOC_CreateTargetControl(rdcstr(), last, rdcstr("rd-trigger"), true);
  if (!tc || !tc->Connected()) {
    fprintf(stderr, "rd-trigger: could not connect to target %u\n", last);
    if (tc) tc->Shutdown();
    RENDERDOC_ShutdownReplay();
    return 1;
  }
  fprintf(stderr, "rd-trigger: connected to %s (pid %u)\n", tc->GetTarget().c_str(), tc->GetPID());
  // On connect the target replays a NewCapture message for each capture it already took; note them
  // so a second trigger in the same run reports only the new capture. Read back to back (one message
  // per call) and keep going until no NewCapture has come for 500 ms, so a long backlog is not
  // cut off and later reported as new.
  std::set<uint32_t> earlier;
  for (auto drain = std::chrono::steady_clock::now() + std::chrono::milliseconds(500);
       std::chrono::steady_clock::now() < drain && tc->Connected();) {
    TargetControlMessage msg = tc->ReceiveMessage(nullptr);
    if (msg.type == TargetControlMessageType::Noop) { sleep_s(0.05); continue; }
    if (msg.type == TargetControlMessageType::NewCapture) {
      earlier.insert(msg.newCapture.captureId);
      drain = std::chrono::steady_clock::now() + std::chrono::milliseconds(500);
    }
  }
  tc->TriggerCapture(frames);

  int rc = 1;
  auto end = std::chrono::steady_clock::now() + std::chrono::duration<double>(wait);
  while (std::chrono::steady_clock::now() < end && tc->Connected()) {
    TargetControlMessage msg = tc->ReceiveMessage(nullptr);
    if (msg.type == TargetControlMessageType::NewCapture && !earlier.count(msg.newCapture.captureId)) {
      printf("%s\n", msg.newCapture.path.c_str());
      rc = 0;
      if (--frames == 0) break;
    } else if (msg.type == TargetControlMessageType::Disconnected) {
      break;
    }
    sleep_s(0.05);
  }
  if (rc != 0) fprintf(stderr, "rd-trigger: no capture reported within %.0f s\n", wait);
  tc->Shutdown();
  RENDERDOC_ShutdownReplay();
  return rc;
}
