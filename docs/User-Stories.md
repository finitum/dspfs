# User Stories
This document describes a series of user stories for the dspfs.

## Background
We follow the format of [behavior-driven development](http://dannorth.net/whats-in-a-story/). Thus, each scenario is of the form:

---
 **Title** _one line describing the story_

**Narrative**

    As a [role]
     I want [feature]
    So that [benefit]

**Acceptance Criteria,** _presented as scenarios of the form:_

    Scenario 1: Title
    Given [context]
     and  [some more context]...
    When  [event] 
    Then  [outcome]
     and  [another outcome]...
---


### User Stories

#### Story 1: Exchanging files
    As a user of dspfs,  
     I want to see all files other users in my group have.  
    So that I can exchange them

    Scenario 1.1: Simple successful download
        Given that *some* user in the group has the file
         and  I want it
         and  said user is online
        When  I select it to be downloaded 
        Then  the download begins
         and  it is added to my own local filesystem

    Scenario 1.2: Multi-user successful download
        Given that multiple users in the group have the file
         and  I want it
         and  multipile users which have the file are online
        When  I select it to be downloaded 
        Then  the download begins in parallel from all available users.
         and  it is added to my own local filesystem

    Scenario 1.3: Unsuccessful download
        Given that no user in the group has the file
         and  I want the file
        When  I try to find it
        Then  the application makes clear nobody has the file
         and  I won't be able to download it

    Scenario 1.4: Pending download
        Given that some user in the group has the file
         and  This user is not online
         and  I want it
        When  I select it to be downloaded 
        Then  the download stays pending
         and  when a user with the file comes online
         and  they still have the file it will be downloaded

    Scenario 1.5: Unsuccessful download due to deleted file
        Given that some user in the group has the file
        and   This user is not online
        and   I want it
        When  I select it to be downloaded 
        Then  the download stays pending
        and   when a user with the file comes online
        and   they don't have the file anymore, the download will fail.
        

#### Story 2: Adding to folder group