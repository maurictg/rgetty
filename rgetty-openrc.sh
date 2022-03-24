#!/usr/bin/openrc-run
export TTY="6"
description="Start rust-getty"
supervisor=supervise-daemon
port="${RC_SVCNAME#*.}"
respawn_period="${respawn_period:-60}"
term_type="${term_type:-linux}"
command="/usr/bin/rgetty"
pidfile="/run/${RC_SVCNAME}.pid"

depend() {
	after local
	keyword -prefix
	provide rgetty
}