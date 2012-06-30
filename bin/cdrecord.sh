#!/bin/sh

ISO=""
DIR=""
BLANK=""
LABEL="$(date)"
DEV="0,0,0"
SPEED="52"

while getopts "c:l:d:i:s:e:" arg
do
	case "$arg" in
		c) BLANK="$OPTARG";;
		l) LABEL="$OPTARG";;
		d) DIR="$OPTARG";;
		i) ISO="$OPTARG";;
		s) SPEED="$OPTARG";;
		e) DEV="$OPTARG";;
		?) printf "Usage: %s: [-c clear [help | all | fast | track | unreserve | trtail | unclose | session]] [-l label [$(date)]] [-d dir]] [-i iso [has the highest priority]] [-s speed [$SPEED]] [-e dev [$DEV]]\n" "$0" 1>&2 && exit 1;;
	esac
done
shift "$(echo "$OPTIND" - 1 | bc)"

if [ -z "$BLANK" ]; then
	if [ -z "$ISO" ]; then
		mkisofs -J -r -v -V "$LABEL" "$DIR" | cdrecord -v dev="$DEV" speed="$SPEED" -driveropts=burnfree -
	else
		cdrecord -v dev="$DEV" speed="$SPEED" -driveropts=burnfree "$ISO"
	fi
else
	cdrecord -v dev="$DEV" speed="$SPEED" blank="$BLANK"
fi
