# TODO: replace with better commands
run:
	cargo run --release

dev0:
	@jq -r 'select(.index == 0) | "\(.timestamp) \(.memory_used)"' gpu_log.jsonl

clean:
	rm -f gpu_log.jsonl

data:
	@jq -r 'select(.index == 0) | "\(.timestamp) \(.memory_used)"' gpu_log2.jsonl > data.txt

plot:
	gnuplot -e "set terminal dumb; plot 'data.txt' with lines"