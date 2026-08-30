#!/bin/bash
G=/home/emoore/CIRISHolon/conformance/gravity
while [ ! -f $G/local2.DONE ]; do sleep 30; done
cd $G && python3 -u closure3.py --c3 > closure3_run.log 2>&1
echo $? > closure3.DONE
