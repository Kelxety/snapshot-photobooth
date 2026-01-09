import Database from '@tauri-apps/plugin-sql';

let db: Database | null = null;

export async function initDatabase() {
  if (!db) {
    db = await Database.load(
      'sqlite:snapbooth.db'
    );
  }
  return db;
}

export interface Event {
  id?: number;
  name: string;
  date: string;
  time: string;
  location?: string;
  description?: string;
  max_photos?: number;
  paper_size?: string;
  template_image?: string;
  photo_boxes?: string;
  created_at?: string;
}

export interface Photo {
  id?: number;
  event_id: number;
  file_path: string;
  taken_at?: string;
}

export interface Share {
  id?: number;
  photo_id: number;
  type:
    | 'email'
    | 'sms'
    | 'facebook'
    | 'twitter'
    | 'print'
    | 'upload';
  status?: 'pending' | 'completed' | 'failed';
  destination?: string;
  created_at?: string;
}

// Event operations
export async function createEvent(
  event: Event
): Promise<number> {
  const database = await initDatabase();
  const result = await database.execute(
    'INSERT INTO events (name, date, time, location, description, max_photos, paper_size, template_image, photo_boxes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)',
    [
      event.name,
      event.date,
      event.time,
      event.location || null,
      event.description || null,
      event.max_photos || 50,
      event.paper_size || '4R',
      event.template_image || null,
      event.photo_boxes || null,
    ]
  );
  return result.lastInsertId ?? 0;
}

export async function getEvent(
  id: number
): Promise<Event | null> {
  const database = await initDatabase();
  const result = await database.select<Event[]>(
    'SELECT * FROM events WHERE id = ?',
    [id]
  );
  return result.length > 0 ? result[0] : null;
}

export async function getAllEvents(): Promise<
  Event[]
> {
  const database = await initDatabase();
  return await database.select<Event[]>(
    'SELECT * FROM events ORDER BY created_at DESC'
  );
}

export async function updateEvent(
  id: number,
  event: Partial<Event>
): Promise<void> {
  const database = await initDatabase();
  const fields: string[] = [];
  const values: any[] = [];

  if (event.name !== undefined) {
    fields.push('name = ?');
    values.push(event.name);
  }
  if (event.date !== undefined) {
    fields.push('date = ?');
    values.push(event.date);
  }
  if (event.time !== undefined) {
    fields.push('time = ?');
    values.push(event.time);
  }
  if (event.location !== undefined) {
    fields.push('location = ?');
    values.push(event.location);
  }
  if (event.description !== undefined) {
    fields.push('description = ?');
    values.push(event.description);
  }
  if (event.max_photos !== undefined) {
    fields.push('max_photos = ?');
    values.push(event.max_photos);
  }

  if (fields.length > 0) {
    values.push(id);
    await database.execute(
      `UPDATE events SET ${fields.join(
        ', '
      )} WHERE id = ?`,
      values
    );
  }
}

export async function deleteEvent(
  id: number
): Promise<void> {
  const database = await initDatabase();
  await database.execute(
    'DELETE FROM events WHERE id = ?',
    [id]
  );
}

// Photo operations
export async function addPhoto(
  photo: Photo
): Promise<number> {
  const database = await initDatabase();
  const result = await database.execute(
    'INSERT INTO photos (event_id, file_path) VALUES (?, ?)',
    [photo.event_id, photo.file_path]
  );
  return result.lastInsertId ?? 0;
}

export async function getEventPhotos(
  eventId: number
): Promise<Photo[]> {
  const database = await initDatabase();
  return await database.select<Photo[]>(
    'SELECT * FROM photos WHERE event_id = ? ORDER BY taken_at DESC',
    [eventId]
  );
}

export async function getPhotoCount(
  eventId: number
): Promise<number> {
  const database = await initDatabase();
  const result = await database.select<
    { count: number }[]
  >(
    'SELECT COUNT(*) as count FROM photos WHERE event_id = ?',
    [eventId]
  );
  return result[0]?.count || 0;
}

// Share operations
export async function createShare(
  share: Share
): Promise<number> {
  const database = await initDatabase();
  const result = await database.execute(
    'INSERT INTO shares (photo_id, type, status, destination) VALUES (?, ?, ?, ?)',
    [
      share.photo_id,
      share.type,
      share.status || 'pending',
      share.destination || null,
    ]
  );
  return result.lastInsertId ?? 0;
}

export async function getEventShares(
  eventId: number
): Promise<any[]> {
  const database = await initDatabase();
  return await database.select(
    `SELECT s.*, p.event_id 
     FROM shares s 
     JOIN photos p ON s.photo_id = p.id 
     WHERE p.event_id = ? 
     ORDER BY s.created_at DESC`,
    [eventId]
  );
}

export async function getShareStats(
  eventId: number
): Promise<any> {
  const database = await initDatabase();
  const stats = await database.select(
    `SELECT 
      s.type,
      COUNT(*) as total,
      SUM(CASE WHEN s.status = 'completed' THEN 1 ELSE 0 END) as completed,
      SUM(CASE WHEN s.status = 'pending' THEN 1 ELSE 0 END) as pending
     FROM shares s
     JOIN photos p ON s.photo_id = p.id
     WHERE p.event_id = ?
     GROUP BY s.type`,
    [eventId]
  );
  return stats;
}

export async function updateShareStatus(
  id: number,
  status: 'pending' | 'completed' | 'failed'
): Promise<void> {
  const database = await initDatabase();
  await database.execute(
    'UPDATE shares SET status = ? WHERE id = ?',
    [status, id]
  );
}
