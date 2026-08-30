#!/bin/bash
G=/home/emoore/CIRISHolon/conformance/gravity
while [ ! -f $G/local1.DONE ]; do sleep 30; done
cd $G && python3 -u closure2.py > closure2_run.log 2>&1
echo $? > closure2.DONE
