Use exactly two Bash tool invocations, then stop.

1. First run only:
   `cd test-harness/hooks/claude`

2. Then run a second Bash command that proves the working directory changed and
   reads a local file from that directory:
   `pwd && sed -n '1,5p' README.md`

After the second command finishes, stop.
