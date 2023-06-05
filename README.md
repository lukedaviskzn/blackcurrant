# Blackcurrant Reception Management System

![Screenshot from 2023-06-05 20-39-00](https://github.com/lukedaviskzn/blackcurrant/assets/18900683/ec32290a-4234-4afc-bed6-4ed36e59d302)

Blackcurrant is a reception management system, intended for record keeping in UCT residence receptions.

The program primarily manages 4 types of records (keys, parcels, games, and items) which can be signed in and out.
Keys, games, and items are limited to a user-defined list. Removing an item from the list does not invalidate old records.
Records are permanent and cannot be edited after creation, with the exception of the notes column, and certain fields when 
signed in/out. No key may be signed out twice at the same time, nor can more games be signed out than are in stock.

Records are stored in a local SQLite DB, thus Blackcurrant can work during loadshedding or internet outage. Automatic 
online backups are planned but not yet implemented. A manual backup of the DB can be saved as an sqlite file.
Records can be exported to CSV if needed for external use.

Currently, no authentication system as not needed in original use case. May be implemented in future.
