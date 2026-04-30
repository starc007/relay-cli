Output demo.mp4

Set Shell "zsh"
Set FontSize 20
Set Width 1400
Set Height 750
Set Theme "Dracula"
Set Padding 24
Set TypingSpeed 65ms
Set PlaybackSpeed 1

# Check ETH price
Type "relay price ETH --chain 1"
Sleep 300ms
Enter
Sleep 2.5s

# Get a quote first
Type "relay quote --from-chain 42161 --from-currency ETH --to-chain 8453 --to-currency ETH --amount 0.001 --user 0x3318D69069846380CEE4A1c390b65Dcf2c7A592E"
Sleep 300ms
Enter
Sleep 3.5s

# Execute the bridge
Type "relay bridge --from-chain 42161 --from-currency ETH --to-chain 8453 --to-currency ETH --amount 0.001 --user 0x3318D69069846380CEE4A1c390b65Dcf2c7A592E"
Sleep 300ms
Enter
Sleep 15s

# Check history
Type "relay history --user 0x3318D69069846380CEE4A1c390b65Dcf2c7A592E"
Sleep 300ms
Enter
Sleep 3s

Sleep 1.5s
