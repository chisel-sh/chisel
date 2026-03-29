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
Type "rm -rf .chisel specs"
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

# 2. Explore Specs
Type "chisel spec"
Sleep 500ms
Enter
Sleep 1.5s

# Navigate specs
Type "j"
Sleep 200ms
Type "j"
Sleep 500ms

# Back to shell
Type "q"
Sleep 500ms

# 3. Machine Mode Output
Type "chisel --machine spec list"
Sleep 500ms
Enter
Sleep 3s
