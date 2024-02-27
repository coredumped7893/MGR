# ./monitor_cpu_usage.sh <pid>
pidstat -h -r -u -v -p $1 5