import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IServer extends Document {
  name: string;
  ownerId: Types.ObjectId;
  members: Types.ObjectId[];
  createdAt: Date;
}

const ServerSchema: Schema<IServer> = new Schema({
  name: { type: String, required: true, unique: true, trim: true },
  ownerId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  members: [{ type: Schema.Types.ObjectId, ref: 'User' }],
  createdAt: { type: Date, default: Date.now }
});

export default (mongoose.models.Server as Model<IServer>) || mongoose.model<IServer>('Server', ServerSchema); 