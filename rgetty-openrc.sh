#!/usr/bin/openrc-run
export TTY="6"
export ERRORDELAY="1000"
export CLEARDELAY="2000"
export ART="/home/yourusername/custom_ascii_art.txt"

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