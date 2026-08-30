#!/bin/bash
G=/home/emoore/CIRISHolon/conformance/gravity
while [ ! -f $G/bridge7.DONE ]; do sleep 30; done
cd $G && python3 -u wilson1.py > wilson1_run.log 2>&1
echo $? > wilson1.DONE
