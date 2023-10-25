# Blackcurrant Reception Management System

![Screenshot of Blackcurrant in operation.](https://github.com/lukedaviskzn/blackcurrant/assets/18900683/d80cddce-028e-4932-a947-f0328c6d257f)

Blackcurrant is a reception management system, intended for record keeping in UCT residence receptions.

The program primarily manages 4 types of records (keys, parcels, games, and items) which can be signed in and out.
Keys, games, and items are limited to a user-defined list. Removing an item from the list does not invalidate old records.
Records are permanent and cannot be edited after creation, with the exception of the notes column, and certain fields when 
signed in/out. No key may be signed out twice at the same time, nor can more games be signed out than are in stock.

Records are stored in a local SQLite DB, thus Blackcurrant can work during loadshedding or internet outage.
Manual backups of the DB can be saved as sqlite files. Records can also be exported to CSV.
