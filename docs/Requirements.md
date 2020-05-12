# dspfs MoSCoW

## Must have

* Sparse checkout: Allows partial checkout of a folder (not downloading all files only some) 
* Distributed: Allow downloading the files from anyone that has "joined" a folder; no central database.
* Folder groups: Everyone in a group can see the same files on any of the computers in the group
* E2EE: All connections should be properly encrypted
* Local per-user db: Should keep track of remote files on a per-user 

## Should have

* NAT Traversal
* Discovery: you should be able to discover local instances without manually specifying IPs

## Could have

* Rate limiting
* Bandwith limiting
* Share files cross groups
* Distributed database with consensus.
* Autodownload: Enables downloading all files in the distributed filesystem

## Won't have

* not tokio
