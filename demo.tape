# Chisel Demo Tape
# Run with: vhs demo.tape

Output apps/landing/public/assets/demo.webm

Set FontSize 16
Set Width 1200
Set Height 800
Set Padding 20
Set WindowBar "Colorful"
Set Theme "Catppuccin Mocha"

# Setup (Hidden)
Hide
Type "rm -rf .chisel"
Enter
Type "chisel init --name 'Chisel Preview'"
Enter
Sleep 1s
Type "clear"
Enter
Show

# 1. Explore Docs
Type "chisel docs"
Sleep 500ms
Enter
Sleep 2s

# Navigate and View
Type "j"
Sleep 200ms
Enter
Sleep 500ms
Type "j"
Sleep 200ms
Type "l"
Sleep 500ms
Type "jjj"
Sleep 1s

# Back to shell
Type "q"
Sleep 500ms

# 2. Explore Issues (Kanban)
Type "chisel issues"
Sleep 500ms
Enter
Sleep 1.5s

# Move Issue Status
Type "j"
Sleep 200ms
Type "m"
Sleep 500ms
Down
Sleep 200ms
Enter
Sleep 1s

# Back to shell
Type "q"
Sleep 500ms

# 3. Machine Mode Output
Type "chisel issues list --machine"
Sleep 500ms
Enter
Sleep 3s
