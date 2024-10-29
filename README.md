# Instructions

Clone this repo into one of your own private repo's. You need to add the reviewer @GarethJamesLarkan to your repo for reviewing. 

For each issue found during your audit, create an issue in the github repo with a detailed explanation of the issue found as well as whether you would flag it as Critical, High, Medium, Low or Informational. The issues will be reviewed by your reviewers.

You DO NOT need to write a POC for the bugs.

# System Design

The system is meant to be a simple NFT purchasing contract where users purchase NFTs with USDC. The only catch is that when a users clicks buy nft, a random number between 1-1000 should be generated and that is how many USDC the user will need to pay.