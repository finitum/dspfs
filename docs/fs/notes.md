Per user the filelist
User : HashMap<UUID,File>


    files:
    Victor:
    dira
        fileb.txt uuida
        
    Jonathan
    dira
        filea.txt uuida
        
    ls /:
        dira:
            fileb.txt duplicate (option> rename)
            filea.txt 
        
### resolve conflict:
* keep mine (do nothing, keep conflict)
* keep theirs (overwrite mine, resolves conflict)
* rename (keep both, resolves conflict)
       

# Requests
## File download request
1. Hash
2. Offset
3. size
* Returns Vec\<u8\> (streaming?)

## Group
### Shared Group
   * map<PathBuf, File>
### Local Group
   * map<PathBuf, File>
   * map<Hash, File>

## Files
### Shared File

### Local File