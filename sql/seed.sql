USE osede_db;

INSERT INTO users (username, password_md5, email, display_name, role) VALUES
('ulla',  '20e59b65186fd11108829062d5ba2968', 'ulla.lindqvist@osedekommun.se', 'Ulla Lindqvist', 'staff'),
('admin', '36f97e92ef3b4c1c6a41e46cc95db899', 'snahldjap@osedekommun.se', 'Snåhl Djåp', 'admin'),
('bengt', 'bf16f908d8ac29459f385599ac58f804', 'bengt.akesson@osedekommun.se', 'Bengt Åkesson', 'staff'),
('karin', '5d7dff2525485995bfe57e5b91de82ee', 'karin.bjork@osedekommun.se', 'Karin Björk', 'staff');

INSERT INTO news (title, body, image, author_id, created_at) VALUES
('Bevattningsförbud!', 'Vi vill härmed påminna att det är strikt förbjudet att vattna trädgård, tvätta kläder, och annat, och helst faktiskt inte ens dricka från kranen. Det är inte särskilt mycket brist på vatten men kommundirektör Snåhl Djåp ska ha poolparty till helgen och behöver hinna fylla alla poolerna tills dess. Vi är tacksamma för att ni visar hänsyn till detta', 'uploads/news_c753e67ce6f48ce25ba1646ec98f84b8.svg', 1, '2026-07-14 08:12:00'),
('Offentlig upphandling: ny pool till kommunhuset', 'Kommunen inleder nu en offentlig upphandling av entreprenad för byggnad av en ny inomhuspool anlagd vid kommunhusets norra fasad. Anbud ska vara upphandlingsenheten senast 15 september. Företrädesrätt till poolen har enligt gällande ordning kommunens tjänsteman med högst lön.', 'uploads/news_92acf3d415454d524a82b3afd937435b.svg', 2, '2026-08-02 09:45:00'),
('Kattegräns införs på Nygatan', 'Från och med augusti gäller kattegräns på Nygatan. Katter som vistas utomhus på gatan utan sin ägare i närhet kommer att konfiskeras och föreviga i kommunens kattearkiv. Ägare kan hämta ut sin katte mot intyg av god sed.', 'uploads/news_cf5bb0f58930eca45edfff8e00a7b37d.svg', 1, '2026-08-05 14:20:00'),
('Ledig tjänst: Poolvakare (säsongsanställning)', 'Kommunen söker en poolvakare för säsongen. Sökanden ska kunna hålla andan i minst fyra minuter och ha inga meningsklotter i tal. Sökanden som inte kan läsa kan ansöka muntligt hos Ulla.', 'uploads/news_09bd061ffd149b7d98eb8f2453d7f101.svg', 1, '2026-08-08 11:03:00'),
('Hundägare påminns', 'Kommunen påminner hundägare om att hundar på allmän plats ska hållas kopplade, särskilt när de springer efter skolan som mestadels inte finns. Hundägaren är också ansvarig för att hålla hunden borta från poolen, kommunens planer och egenkänslor. Vid upprepade tillfällen kan kommunen förelägga hundägaren att lära hunden kommunens ordning.', 'uploads/news_ed1b60b07a696da4fd735db1f08a2b27.svg', 1, '2026-08-09 13:35:00'),
('Vägsankning mellan Tången och Backen', 'Vägen mellan Tången och Backen sankas på löpande basis. Trafikanter uppmanas att gå. Större hål kan anmälas till väghållaren som mottar klagomål varje torsdag kl 09:00 vid skyltverket.', 'uploads/news_df4e7f8b35b122187c96fa1675057abc.svg', 1, '2026-08-11 07:55:00'),
('Biblioteket övergår i vintertid', 'Från och med 1 oktober går biblioteket över i vintertid. Biblioteket har öppettider på onsdagar och fredagar. Lånetiden för böcker om kommunens historia förlängs till tre veckor eftersom de är eftertraktade.', 'uploads/news_7b4573e85ae6b7fc81a895e9c0b038a4.svg', 1, '2026-08-15 16:40:00'),
('Snåhl Djåp nominerad till Årets Poolgäst', 'Kommunens poolgäster har röstat fram Snåhl Djåp som nominerad till Årets Poolgäst. Motiveringen lyder att ingen annan gäst badar lika tyst. Priset utdelas vid årets avstämning.', 'uploads/news_8c91640877e8a476831ffe8acbc41d0a.svg', 1, '2026-08-19 10:15:00');

INSERT INTO notes (id, user_id, title, body, created_at) VALUES
(140, 1, 'Inköp', 'Får, bröd, mjölk, kaffe, papper till skrivaren som inte fungerar.', '2026-08-20 08:30:00'),
(143, 1, 'Fönster', 'Kontorsfönster mot gården måste tvättas. Få inte tag i någonting som når ut tillräckligt. Kanske bengt kan låna steget.', '2026-08-22 13:10:00'),
(147, 1, 'Post', 'Kolla postlådan. Det låg ett brev från Skatteverket igen. Det brukar vara skicka.', '2026-08-25 09:45:00'),
(151, 1, 'Poolen', 'Vattentemperaturen i poolen blev 19 grader i torsdags. Det var inte okej. Ring poolen i morgon.', '2026-08-28 17:25:00'),
(156, 1, 'Kaffebryggare', 'Kaffebryggaren i köket läcker. Anmäl till fastighetsskötaren. Om han inte fixar det så är det kanske dags för en ny.', '2026-09-01 12:00:00'),
(159, 2, 'Lösenord', 'you know what it is Elessar, Elfstone, Strider, heir of Isildur, ancient king of Arnor and Gondor', '2026-09-03 21:14:00'),
(164, 2, 'Poolparty', 'Planera poolparty. Bjud in alla. Låta ulla ordna fika. Kolla att poolen inte är för kall.', '2026-09-05 15:40:00'),
(168, 2, 'Kaffebryggare', 'Ulla säger att kaffebryggaren är trasig igen. Beställ en ny. Be bengt kolla var de sparar fakturan.', '2026-09-08 10:05:00');

INSERT INTO messages (from_id, to_id, subject, body, preview, created_at, is_read) VALUES
(2, 1, 'Poolparty i veckan?', 'Hej Ulla! Jag har bestämt mig. Vi kör poolparty på fredag. Du ordnar fikat och jag ordnar att poolen blir varm. Svara så att jag vet att du såg detta.', 'Hej Ulla! Jag har bestämt mig. Vi kör poolparty på fredag. Du ordnar fikat och jag ordnar att poolen blir varm.', '2026-09-07 08:20:00', 1),
(1, 2, 'RE: Poolparty i veckan?', 'Klart! Jag bakar kanelbullar. Men varna personalen i förväg, annars kommer de för tidigt och badar ur värmen.', 'Klart! Jag bakar kanelbullar. Men varna personalen i förväg, annars kommer de för tidigt och badar ur värmen.', '2026-09-07 09:05:00', 1),
(1, 1, 'Till mig själv', 'Kom ihåg att lämna in löneunderlaget fredag. Och kom ihåg att stänga fönstret i köket, det är kallt på natten.', 'Kom ihåg att lämna in löneunderlaget fredag. Och kom ihåg att stänga fönstret i köket, det är kallt på natten.', '2026-09-08 16:30:00', 1),
(2, 1, 'Hålet på vägen', 'Bengt anmälde ett stort hål på vägen till Backen igen. Kanske du kan skriva till väghållaren att de faktiskt ska fixa det och inte bara sankar igen.', 'Bengt anmälde ett stort hål på vägen till Backen igen. Kanske du kan skriva till väghållaren att de faktiskt ska fixa det och inte bara sankar igen.', '2026-09-10 11:15:00', 1),
(4, 2, 'Biljetter till avstämningen', 'Snåhl! Jag har sparat två biljetter till avstämningen åt dig, de ligger hos ulla. Hittar du den här länken så slipper du fråga efter dem: http://osedekommun.se/biljetter', 'Snåhl! Jag har sparat två biljetter till avstämningen åt dig, de ligger hos ulla. Hittar du den här länken så slipper du fråga efter dem.', '2026-09-12 14:50:00', 0);
