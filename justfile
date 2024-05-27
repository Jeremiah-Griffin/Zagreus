set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
default:
	just -l

alias t := test
test:
	cargo t -- --all-features
